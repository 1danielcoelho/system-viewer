import os
import re
import glob
import numpy as np

folder_path = r"horizons_ephemeris_heliocentric/planets_and_moons/*.txt"
out_path = r"../public/body_data/heliocentric/planets_moons.gen.csv"

target_body_name_re = re.compile(r"Target body name: ([^;]+?) \(")
center_body_name_re = re.compile(r"Center body name: ([^;]+?) \(")
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

curr_dir = os.path.dirname(os.path.realpath(__file__))
folder_path = os.path.join(curr_dir, folder_path)
out_path = os.path.join(curr_dir, out_path)


class Body:
    def __init__(self):
        self.id = 0
        self.ref = 0
        self.name = ''
        self.mean_radius = 2.0
        self.eccentricity = 0.0
        self.periapsis_distance = 0.0
        self.inclination = 0.0
        self.long_asc_node = 0.0
        self.arg_periapsis = 0.0
        self.time_of_periapsis = 0.0
        self.mean_motion = 0.0
        self.mean_anomaly = 0.0
        self.true_anomaly = 0.0
        self.semi_major_axis = 0.0
        self.apoapsis_distance = 0.0
        self.sidereal_orbit_period = 0.0
        self.type = ""

    def __str__(self):
        return ",".join([str(x) for x in [
            self.id,
            self.name,
            self.ref,
            self.type,
            self.semi_major_axis,
            self.eccentricity,
            self.inclination,
            self.long_asc_node,
            self.arg_periapsis,
            self.mean_anomaly,
            self.mean_radius,
        ]])

    def __repr__(self):
        return self.__str__()


orig_names = glob.glob(folder_path)

sun = Body()
sun.id = 0
sun.ref = 0
sun.name = 'Sun'
sun.mean_radius = 695500.0

bodies = {sun.id: sun}

for filename in orig_names:
    name_no_ext = re.findall(r"([^\\]+)\.txt", filename)[0]

    body_id = name_no_ext[:name_no_ext.find('@')]
    reference = name_no_ext[name_no_ext.find('@')+1:]

    if reference == 'sun':
        reference = 0

    with open(filename) as f:
        data = f.read()

        new_body = Body()
        new_body.id = int(body_id)
        new_body.ref = int(reference)
        new_body.name = re.findall(target_body_name_re, data)[0]

        # Try and get a decent mean radius. If not, it will be left at 2 km
        radius = re.findall(mean_radius_re, data)
        if radius and radius[0] is not ' ':
            radius = radius[0]
            radii = [float(val.strip()) for val in radius.split('x')]
            radius = np.mean(radii)
            new_body.mean_radius = radius

        # Clip everything before and after $$SOE and $$EOE, since it sometimes contains
        # things that trip our re
        data = re.split(r'\$\$SOE|\$\$EOE', data)[1]

        try:
            new_body.eccentricity = float(re.findall(eccentricity_re, data)[0])
            new_body.periapsis_distance = float(re.findall(periapsis_distance_re, data)[0])
            new_body.inclination = float(re.findall(inclination_re, data)[0])
            new_body.long_asc_node = float(re.findall(long_asc_node_re, data)[0])
            new_body.arg_periapsis = float(re.findall(arg_periapsis_re, data)[0])
            new_body.time_of_periapsis = float(re.findall(time_of_periapsis_re, data)[0])
            new_body.mean_motion = float(re.findall(mean_motion_re, data)[0])
            new_body.mean_anomaly = float(re.findall(mean_anomaly_re, data)[0])
            new_body.true_anomaly = float(re.findall(true_anomaly_re, data)[0])
            new_body.semi_major_axis = float(re.findall(semi_major_axis_re, data)[0])
            new_body.apoapsis_distance = float(re.findall(apoapsis_distance_re, data)[0])
            new_body.sidereal_orbit_period = float(re.findall(sideral_orbit_period_re, data)[0])
        except BaseException as e:
            print(data)
            raise

        bodies[new_body.id] = new_body

# Add radius of pluto manually since that seems to be different than all the others
bodies[999].mean_radius = 1195

# Fake add mercury and venus, since they have no satellites and the "barycenter files" for them is themselves
mercury_bary = bodies[1]
venus_bary = bodies[2]
mercury_body = Body()
mercury_body.ref = 1
mercury_body.id = 199
mercury_body.name = 'Mercury'
mercury_body.mean_radius = mercury_bary.mean_radius
bodies[199] = mercury_body
venus_body = Body()
venus_body.ref = 2
venus_body.id = 299
venus_body.name = 'Venus'
venus_body.mean_radius = venus_bary.mean_radius
bodies[299] = venus_body

# Add body types
for body in bodies.values():
    if body.id == 0:
        body.type = "star"
    elif body.id > 100 and (body.id + 1) % 100 == 0:
        body.type = "planet"
    elif body.ref == 0 and body.id < 10:
        body.type = "system barycenter"
    else:
        body.type = "satellite"

# Convert all distances to Mm
for body in bodies.values():
    # If its a planet-system barycenter, set a zero radius
    if 0 < body.id < 10:
        body.mean_radius = 0

    body.mean_radius /= 1000.0
    body.periapsis_distance *= 149597.8707
    body.semi_major_axis *= 149597.8707
    body.apoapsis_distance *= 149597.8707

# Create a list in the best order for display
# Sun, planet-bary, planet, moons, planet-bary, planet, moons, etc
sorted_keys = sorted(bodies.keys())
sorted_bodies = []
sorted_bodies.append((sun.id, sun))
del bodies[sun.id]
for i in range(1, 10):
    # Bary
    sorted_bodies.append((i, bodies[i]))
    del bodies[i]

    # Planet
    planet_index = i*100 + 99
    sorted_bodies.append((planet_index, bodies[planet_index]))
    del bodies[planet_index]

    # Moons
    for id in sorted_keys:
        system = int(str(id)[0])
        if i != system:
            continue

        if id in bodies:
            sorted_bodies.append((id, bodies[id]))
            del bodies[id]


for (body_id, body) in sorted_bodies:
    try:
        calc_tp = 2451545.0 - (body.mean_anomaly - 360) / body.mean_motion
        print(body_id, calc_tp, body.time_of_periapsis, round((calc_tp - body.time_of_periapsis) / body.sidereal_orbit_period, 3), body.mean_anomaly > 180.0)
    except ZeroDivisionError:
        pass

# with open(out_path, 'w') as f:
#     print("Writing", str(len(sorted_bodies)), "bodies to", out_path)
#     f.write("\n".join([str(entry[1]) for entry in sorted_bodies]))
#     print("Done")
