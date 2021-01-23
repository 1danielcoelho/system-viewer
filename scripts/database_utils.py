import os
import sys
import json
import copy
import re

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


def load_database():
    """ Read existing files and load into maps """
    database = {}
    for file in files:
        database[file] = {}
        path = os.path.join(database_folder, file + ".json")
        if os.path.exists(path):
            with open(path, "r") as f:
                database[file] = json.load(f)
    return database


def get_body_by_name(database, name):
    for base in database.values():
        for body in base.values():
            if body['name'] == name:
                return body

    return None


def handle_database_exceptions(database):
    """ Handle annoying exceptions before saving the database
    Like making sure that Mercury/Venus have a separate entry for barycenter, etc.
    """

    fake_osc_elements = {
        "epoch": 2451545.0,
        "ref_id": "10",
        "eccentricity": 1,
        "periapsis_distance": 0,
        "inclination": 0,
        "long_asc_node": 0,
        "arg_periapsis": 0,
        "time_of_periapsis": 2451545.0,
        "mean_motion": 0,
        "mean_anomaly": 0,
        "true_anomaly": 0,
        "semi_major_axis": 0,
        "apoapsis_distance": 0,
        "sidereal_orbit_period": 1,
    }

    # Sun shouldn't have gotten any osc_elements yet since all our elements are heliocentric
    try:
        database['major_bodies']['10']['osc_elements'] = [fake_osc_elements]
    except KeyError:
        pass

    # Mercury Barycenter is the same as Mercury, but we kind of want separate entries for consistency
    try:
        database['major_bodies']['1'] = copy.deepcopy(database['major_bodies']['199'])
        database['major_bodies']['1']['name'] = 'Mercury Barycenter'
        database['major_bodies']['1']['type'] = 'barycenter'
        database['major_bodies']['1']['mass'] = 0
        database['major_bodies']['1']['radius'] = 0
        database['major_bodies']['1']['albedo'] = 0
        database['major_bodies']['1']['magnitude'] = 0
        database['major_bodies']['1']['rotation_period'] = 0
        database['major_bodies']['1']['rotation_axis'] = [0, 0, 0]
        del database['major_bodies']['1']['state_vectors']
        database['major_bodies']['199']['name'] = 'Mercury'
        database['major_bodies']['199']['osc_elements'] = [copy.deepcopy(fake_osc_elements)]
        database['major_bodies']['199']['osc_elements'][0]['ref_id'] = '1'
    except KeyError:
        pass

    # Venus Barycenter is the same as Venus, but we kind of want separate entries for consistency
    try:
        database['major_bodies']['2'] = copy.deepcopy(database['major_bodies']['299'])
        database['major_bodies']['2']['name'] = 'Venus Barycenter'
        database['major_bodies']['2']['type'] = 'barycenter'
        database['major_bodies']['2']['mass'] = 0
        database['major_bodies']['2']['radius'] = 0
        database['major_bodies']['2']['albedo'] = 0
        database['major_bodies']['2']['magnitude'] = 0
        database['major_bodies']['2']['rotation_period'] = 0
        database['major_bodies']['2']['rotation_axis'] = [0, 0, 0]
        del database['major_bodies']['2']['state_vectors']
        database['major_bodies']['299']['name'] = 'Venus'
        database['major_bodies']['299']['osc_elements'] = [copy.deepcopy(fake_osc_elements)]
        database['major_bodies']['299']['osc_elements'][0]['ref_id'] = '2'
    except KeyError:
        pass


def save_database(database):
    """ Write database back to their individual files """

    handle_database_exceptions(database)

    # Sort all databases
    for db_name in database.keys():
        database[db_name] = {k: v for k, v in sorted(database[db_name].items(), key=lambda item: float(item[0]))}

    # Write database to files
    for filename in database:
        path = os.path.join(database_folder, filename + ".json")
        with open(path, "w") as f:
            json.dump(database[filename], f)
