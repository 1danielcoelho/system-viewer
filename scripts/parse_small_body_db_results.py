import json
import os
import re
import glob
import numpy as np
import copy
from database_utils import save_database, load_database
from constants import *
from math import sqrt

main_file = "D:/Dropbox/Astronomy/asteroid_comet_elements.csv"
spectral_file = "D:/Dropbox/Astronomy/asteroid_comet_spectral.csv"
num_asteroids = 10000
num_comets = 10000
only_with_mass = False  # Only fetch objects whose mass can be estimated


def add_small_body_data(database):
    db_asteroids = database['asteroids']
    db_comets = database['comets']
    db_osc_elements = database['osc_elements']

    count_asteroid = 0
    count_comet = 0
    with open(main_file, "r") as f:
        # Skip header
        f.readline()

        asteroid_count = 0
        comet_count = 0

        while True:
            line = f.readline()
            if not line:
                break

            is_asteroid = line[0] == 'a'
            is_comet = line[0] == 'c'

            vals = line.split(',')
            body_id_str = str(vals[0])
            body_name = vals[1]
            body_gm = vals[4]
            body_diameter = vals[5]
            body_H = vals[6]
            body_albedo = vals[7]
            body_rot_per = vals[8]

            assert(body_id_str)

            body = {}
            body['name'] = body_name
            body['type'] = 'asteroid' if is_asteroid else 'comet'

            if body_H:
                body['magnitude'] = float(body_H)

            if body_albedo:
                body['albedo'] = float(body_albedo)

            if body_diameter:
                body['radius'] = float(body_diameter) / 2000.0

            if body_rot_per:
                body['rotation_period'] = float(body_rot_per) / 24.0

            if body_gm:
                body['mass'] = float(body_gm) / G
            else:
                if only_with_mass:
                    continue

            elements = {}
            elements['ref_id'] = '10'  # All heliocentric
            elements['epoch'] = float(vals[9])  # JDN
            elements['e'] = float(vals[11])
            elements['a'] = float(vals[12]) * au_to_Mm
            elements['i'] = float(vals[14]) * deg_to_rad
            elements['O'] = float(vals[15]) * deg_to_rad
            elements['w'] = float(vals[16]) * deg_to_rad
            elements['M'] = float(vals[17]) * deg_to_rad
            elements['p'] = float(vals[19])

            if is_asteroid:
                asteroid_count += 1
            elif is_comet:
                comet_count += 1

            skip_asteroid = is_asteroid and asteroid_count > num_asteroids
            skip_comet = is_comet and comet_count > num_comets

            if skip_asteroid and skip_comet:
                break
            elif skip_asteroid or skip_comet:
                continue

            if body_id_str not in db_osc_elements:
                db_osc_elements[body_id_str] = []
            db_osc_elements[body_id_str].append(elements)

            if is_asteroid:
                db_asteroids[body_id_str] = body
                count_asteroid += 1
            else:
                db_comets[body_id_str] = body
                count_comet += 1

    count_spectral = 0
    with open(spectral_file, "r") as f:
        # Skip header
        f.readline()

        while True:
            line = f.readline()
            if not line:
                break

            is_asteroid = line[0] == 'a'

            vals = line.split(',')
            body_id = str(vals[0])
            body_name = vals[1]
            body_smassii = vals[2].strip()
            body_tholen = vals[3].strip()

            assert(body_id)

            db = db_asteroids if is_asteroid else db_comets
            try:
                body = db[body_id]

                if body_smassii:
                    body['spec_smassii'] = body_smassii

                if body_tholen:
                    body['spec_tholen'] = body_tholen

                count_spectral += 1

            except KeyError:
                # print(f"Failed to find body with id {body_id}, name '{body_name}' to unload spectral info into")
                pass

    return count_asteroid, count_comet, count_spectral


def run(database):
    print("Parsing HORIZONS small body database dump...")
    count_asteroid, count_comet, count_spectral = add_small_body_data(database)
    print(f"Parsed {count_asteroid} asteroids, {count_comet} comets and spectral data for {count_spectral} of those from HORIZONS small body database dump")


if __name__ == "__main__":
    database = load_database()

    run(database)

    save_database(database)
