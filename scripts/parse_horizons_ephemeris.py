import json
import os
import re
import glob
import numpy as np

horizons_pattern = "D:/Dropbox/Astronomy/horizons_ephemeris_heliocentric/*.txt"
# horizons_pattern = "D:/Dropbox/Astronomy/horizons_statevectors_ssb_j2000/*.txt"

database_folder = "D:/Dropbox/Astronomy/database"
files = [
    "asteroids",
    "comets",
    "jovian_satellites",
    "saturnian_satellites",
    "other_satellites",
    "major_bodies",
    "artificial"
]

target_body_name_re = re.compile(r"Target body name: ([^;]+?) \((\d+)\)")
center_body_name_re = re.compile(r"Center body name: ([^;]+?) \((\d+)\)")
eccentricity_re = re.compile(r" EC=[\s]*([\d\-+eE.]+)")
periapsis_distance_re = re.compile(r" QR=[\s]*([\d\-+eE.]+)")
inclination_re = re.compile(r" IN=[\s]*([\d\-+eE.]+)")
long_asc_node_re = re.compile(r" OM=[\s]*([\d\-+eE.]+)")
arg_periapsis_re = re.compile(r" W =[\s]*([\d\-+eE.]+)")
time_of_periapsis_re = re.compile(r" Tp=[\s]*([\d\-+eE.]+)")
mean_motion_re = re.compile(r" N =[\s]*([\d\-+eE.]+)")
mean_anomaly_re = re.compile(r" MA=[\s]*([\d\-+eE.]+)")
true_anomaly_re = re.compile(r" TA=[\s]*([\d\-+eE.]+)")
semi_major_axis_re = re.compile(r" A =[\s]*([\d\-+eE.]+)")
apoapsis_distance_re = re.compile(r" AD=[\s]*([\d\-+eE.]+)")
sideral_orbit_period_re = re.compile(r" PR=[\s]*([\d\-+eE.]+)")
mean_radius_re = re.compile(r"[R,r]adius[ \t\(\)IAU,]+km[ \t\)=]+([\d.x ]+)")
output_type_re = re.compile(r"Output type\s+:(.*)")
elements_entry_re = re.compile(r"(([\d.]+)[ =,]+A\.D\.[\s\S]+?)(?:(?=[\d.]+[ =,]+A\.D\.)|\Z)")

au_to_Mm = 149597.8707
deg_to_rad = 3.14159265358979323846264 / 180.0

def get_body_database(body_id):
    try:
        body_id = int(body_id)
        if body_id <= 10 or (body_id > 100 and (body_id + 1) % 100 == 0):
            return "major_bodies"
        elif (body_id > 500 and body_id < 599) or (body_id > 55500 and body_id < 55510) or body_id in [55060, 55061, 55062, 55064, 55065, 55066, 55068, 55070, 55071, 55074]:
            return 'jovian_satellites'
        elif (body_id > 600 and body_id < 700) or body_id in [65035, 65040, 65041, 65045, 65048, 65050, 65055, 65056, 65065, 65066, 65067, 65068, 65069, 65070, 65071, 65071, 65073, 65074, 65075, 65076, 65077, 65078]:
            return 'saturnian_satellites'
        elif (body_id > 700 and body_id < 999) or body_id in [301, 401, 402]:
            return 'other_satellites'
    except ValueError:
        pass

    print(body_id)

    if body_id[0] == 'a':
        return 'asteroids'
    elif body_id[0] == 'c':
        return 'comets'
    
    raise ValueError('Unexpected body_id ' + str(body_id))


def get_body_type(body_id):
    try:
        body_id = int(body_id)
        if body_id < 10:
            return 'barycenter'
        if body_id == 10:
            return 'star'
        elif body_id > 100 and (body_id + 1) % 100 == 0:
            return 'planet'
        elif body_id > 100:
            return 'satellite'
    except ValueError:
        pass

    if body_id[0] == 'a':
        return 'asteroid'
    elif body_id[0] == 'c':
        return 'comet'
    
    raise ValueError('Unexpected body_id ' + str(body_id))


# Read existing files and load into maps
database = {}
for file in files:
    database[file] = {}
    path = os.path.join(database_folder, file + ".json")
    if os.path.exists(path):
        with open(path, "r") as f:
            database[file] = json.load(f)

# Parse horizons data and load it
horizon_file_names = glob.glob(horizons_pattern)
# horizon_file_names = ["C:/Users/1dani/Desktop/test.txt"]
for filename in horizon_file_names:
    with open(filename) as f:
        data = f.read()

        name, body_id = re.findall(target_body_name_re, data)[0]
        ref_name, ref_id = re.findall(center_body_name_re, data)[0]

        print(name, body_id, ref_name, ref_id)

        database_name = get_body_database(body_id)
        db = database[database_name]
        
        if body_id not in db:
            db[body_id] = {}
        body_entry = db[body_id]
        
        body_entry['name'] = name
        body_entry['type'] = get_body_type(body_id)
        body_entry['meta'] = {}

        # Radius
        radius = 0.0
        radius_str = re.findall(mean_radius_re, data)
        if radius_str and radius_str[0] is not ' ':
            radius_str = radius_str[0]
            radii = [float(val.strip()) for val in radius_str.split('x')]
            radius = np.mean(radii)
        radius /= 1000.0  # Km to Mm
        body_entry['radius'] = radius

        # Something like ' GEOMETRIC cartesian states' or ' GEOMETRIC osculating elements'
        horizons_output_type = re.findall(output_type_re, data)[0]

        # Clip everything before and after $$SOE and $$EOE, since it sometimes contains
        # things that trip our re
        data = re.split(r'\$\$SOE|\$\$EOE', data)[1]

        # Orbital elements
        if 'osculating elements' in horizons_output_type:            
            if 'osc_elements' not in body_entry:
                body_entry['osc_elements'] = []
            osc_elements = body_entry['osc_elements']
            osc_elements.sort(key=lambda els: els['epoch'])

            all_parsed_elements = []
            entries = re.findall(elements_entry_re, data)
            for entry in entries:
                full_entry = entry[0]

                parsed_elements = {}
                parsed_elements['epoch'] = float(entry[1])
                parsed_elements['ref_id'] = ref_id
                parsed_elements['eccentricity'] = float(re.findall(eccentricity_re, full_entry)[0]) 
                parsed_elements['periapsis_distance'] = float(re.findall(periapsis_distance_re, full_entry)[0]) * au_to_Mm
                parsed_elements['inclination'] = float(re.findall(inclination_re, full_entry)[0]) * deg_to_rad
                parsed_elements['long_asc_node'] = float(re.findall(long_asc_node_re, full_entry)[0]) * deg_to_rad
                parsed_elements['arg_periapsis'] = float(re.findall(arg_periapsis_re, full_entry)[0]) * deg_to_rad
                parsed_elements['time_of_periapsis'] = float(re.findall(time_of_periapsis_re, full_entry)[0])
                parsed_elements['mean_motion'] = float(re.findall(mean_motion_re, full_entry)[0]) * deg_to_rad
                parsed_elements['mean_anomaly'] = float(re.findall(mean_anomaly_re, full_entry)[0]) * deg_to_rad
                parsed_elements['true_anomaly'] = float(re.findall(true_anomaly_re, full_entry)[0]) * deg_to_rad
                parsed_elements['semi_major_axis'] = float(re.findall(semi_major_axis_re, full_entry)[0]) * au_to_Mm
                parsed_elements['apoapsis_distance'] = float(re.findall(apoapsis_distance_re, full_entry)[0]) * au_to_Mm
                parsed_elements['sidereal_orbit_period'] = float(re.findall(sideral_orbit_period_re, full_entry)[0])
                all_parsed_elements.append(parsed_elements)
            
            all_parsed_elements.sort(key=lambda els: els['epoch'])

            for parsed_element in all_parsed_elements:
                parsed_epoch = parsed_element['epoch']

                found = False
                for index, element in enumerate(osc_elements):
                    epoch = element['epoch']
                    
                    if epoch > parsed_epoch:
                        break

                    if epoch == parsed_epoch and element['ref_id'] == parsed_element['ref_id']:
                        osc_elements[index] = parsed_element
                        found = True
                        break
                    
                if not found:
                    osc_elements.append(parsed_element)
        
        elif 'cartesian states' in horizons_output_type:
            if 'state_vectors' not in body_entry:
                body_entry['state_vectors'] = []
            state_vectors = body_entry['state_vectors']
            state_vectors.sort(key=lambda vec: vec[0])

            all_parsed_vectors = []
            entries = re.findall(elements_entry_re, data)
            for entry in entries:
                full_entry = entry[0]

                values = full_entry.strip().split(",")
                epoch = float(values[0])
                xyz_vxvyvz = [float(val) / 1000.0 for val in values[3:] if val]

                parsed_vector = [epoch] + xyz_vxvyvz
                all_parsed_vectors.append(parsed_vector)
            
            all_parsed_vectors.sort(key=lambda vec: vec[0])

            for parsed_vector in all_parsed_vectors:
                parsed_epoch = parsed_vector[0]

                found = False
                for index, vec in enumerate(state_vectors):
                    epoch = vec[0]
                    
                    if epoch > parsed_epoch:
                        break

                    if epoch == parsed_epoch:
                        state_vectors[index] = parsed_vector
                        found = True
                        break
                    
                if not found:
                    state_vectors.append(parsed_vector)


# Write database to files
for filename in database:
    path = os.path.join(database_folder, filename + ".json")
    with open(path, "w") as f:
        json.dump(database[filename], f)
