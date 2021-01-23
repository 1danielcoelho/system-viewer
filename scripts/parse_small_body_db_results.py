import json
import os
import re
import glob
import numpy as np
import copy
from database_utils import save_database, load_database

main_file = "D:/Dropbox/Astronomy/asteroid_comet_elements.csv"
num_asteroids = 1000
num_comets = 1000

au_to_Mm = 149597.8707
deg_to_rad = 3.14159265358979323846264 / 180.0
G = 6.67259E-20  # km3/(s2 kg1)


def add_small_body_data(database):
    db_asteroids = database['asteroids']
    db_comets = database['comets']

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

            vals = line.split(',')
            body_id = vals[0]
            body_name = vals[1]
            body_gm = vals[4]
            body_diameter = vals[5]
            body_H = vals[6]
            body_albedo = vals[7]
            body_rot_per = vals[8]

            assert(body_id)

            body = {}
            body['name'] = body_name
            body['type'] = 'asteroid' if is_asteroid else 'comet'
            
            if body_diameter:
                body['radius'] = float(body_diameter) / 2000.0

            if body_H:
                body['magnitude'] = float(body_H)

            if body_albedo:
                body['albedo'] = float(body_albedo)
            
            if body_rot_per:
                body['rotation_period'] = float(body_rot_per) / 24.0

            if body_gm:
                body['mass'] = float(body_gm) / G

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

            body['osc_elements'] = [elements]
            if is_asteroid:
                db_asteroids[body_id] = body
            else:
                db_comets[body_id] = body


if __name__ == "__main__":
    database = load_database()

    add_small_body_data(database)

    save_database(database)
