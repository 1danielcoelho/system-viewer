from constants import *
from math import sin, cos, sqrt, atan2
from database_utils import save_database, load_database

materials = {
    '199': 'rocky',
    '299': 'atmo',
    '399': 'atmo',
    '499': 'atmo',
    '599': 'gas',
    '699': 'gas',
    '799': 'gas',
    '899': 'gas',
    '999': 'rocky',
}

material_parameters = {
    '10': {
        'base_color': 'FFFFFFFF',
        'emissive_texture': '2k_sun.jpg'
    },
    '199': {
        'base_color': '726658FF',
        'base_color_texture': '2k_mercury.jpg'
    },
    '299': {
        'base_color': 'EFECDDFF',
        'base_color_texture': '2k_venus_atmosphere.jpg'
    },
    '301': {
        'base_color': 'cfcfd1',
        'base_color_texture': '2k_moon.jpg',
    },
    '399': {
        'base_color': 'A49FB3FF',
        'base_color_texture': '2k_earth_daymap.jpg',
        'normal_texture': '2k_earth_normal_map.tif',
        'metal_rough_texture': '2k_earth_metal_rough.tif',
    },
    '499': {
        'base_color': '896545FF',
        'base_color_texture': '2k_mars.jpg'
    },
    '599': {
        'base_color': 'C3BEABFF',
        'base_color_texture': '2k_jupiter.jpg'
    },
    '699': {
        'base_color': 'C9B38EFF',
        'base_color_texture': '2k_saturn.jpg'
    },
    '799': {
        'base_color': 'AED5DAFF',
        'base_color_texture': '2k_uranus.jpg'
    },
    '899': {
        'base_color': '91AFBAFF',
        'base_color_texture': '2k_neptune.jpg'
    },
    '999': {
        'base_color': 'C09F82FF',
    },
}


def add_material_info(database):
    count = 0
    for db_name, db in database.items():
        if db_name in ['state_vectors', 'osc_elements']:
            continue

        for body_id, body in db.items():

            # Material choice
            set_mat = False
            try:
                mat = materials[body_id]
                body['material'] = mat
            except KeyError:
                pass

            # Material parameters
            set_params = False
            try:
                params = material_parameters[body_id]

                if 'material_params' not in body:
                    body['material_params'] = {}

                body['material_params'].update(params)
                set_params = True
            except KeyError:
                pass

            if set_mat or set_params:
                count += 1

    return count


def run(database):
    print("Adding material info to database bodies...")
    count = add_material_info(database)
    print(f"Added material info to {count} database bodies")


if __name__ == "__main__":
    database = load_database()

    run(database)

    save_database(database)
