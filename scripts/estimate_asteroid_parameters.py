import json
import os
import re
import glob
import numpy as np
import copy
from database_utils import save_database, load_database
from math import sqrt

au_to_Mm = 149597.8707
pi = 3.14159265358979323846264
deg_to_rad = pi / 180.0
G = 6.67259E-20  # km3/(s2 kg1)


def estimate_body_data(database):
    # Estimate body radius from magnitude and albedo
    # Source: https://space.stackexchange.com/questions/36/how-can-i-derive-an-asteroid-mass-size-estimate-from-jpl-parameters
    # https://en.wikipedia.org/wiki/Absolute_magnitude#Solar_System_bodies_(H)
    for db in database.values():
        for body_id, body in db.items():            
            if 'radius' not in body and 'magnitude' in body and 'albedo' in body:
                diameter_km = (1329.0 / sqrt(body['albedo'])) * 10 ** (-0.2 * body['magnitude'])
                body['radius'] = diameter_km / 2000.0
                print(f'Estimated radius {body["radius"]} Mm for body "{body_id}" (name "{body["name"]}")')

    # Estimate asteroid mass by using standard densities
    # https://space.stackexchange.com/questions/2882/method-to-estimate-asteroid-density-based-on-spectral-type
    # https://en.wikipedia.org/wiki/Standard_asteroid_physical_characteristics#:~:text=For%20many%20asteroids%20a%20value,and%205.32%20g%2Fcm3.
    db = database['asteroids']
    for body_id, body in db.items():            
        if 'mass' not in body and 'radius' in body:           
            
            density = 2E21 # kg/Mm3 (corresponds to 2 g/cm3)

            # If it has a tholen spectral class we can do a bit better
            if 'spec_tholen' in body:
                tholen_class = body['spec_tholen'].upper()
                for letter in tholen_class: # Can be up to 4, best fitting type mentioned first
                    
                    # Class X is either E, M or P depending on albedo
                    if letter == 'X':
                        try:
                            albedo = body['albedo']
                            if albedo > 0.3:
                                letter = 'E'
                            elif albedo < 0.1:
                                letter = 'P'
                            else:
                                letter = 'M'
                        except KeyError:
                            letter = 'M' # Assume its of the most common subclass

                    if letter in 'CDPTBGF':
                        density = 1.38E21
                        break
                    elif letter in 'SKQVRAE':
                        density = 2.71E21
                        break
                    elif letter == 'M':
                        density = 5.32E21
                        break    
                    elif letter in 'I:': # Inconsitent or noisy data. If this is the best we got just ignore the tholen class
                        break
                    else:
                        print(f"Unexpected tholen class {tholen_class} for body {body_id}")

            volume = 4.0/3.0 * pi * body['radius'] ** 3
            mass = volume * density
            print(f'Estimated mass {mass:E} kg from density {density} kg/Mm3 for body "{body_id}" (name "{body["name"]}")')
            body['mass'] = mass

if __name__ == "__main__":
    database = load_database()

    estimate_body_data(database)

    save_database(database)
