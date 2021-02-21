from constants import *
from math import sin, cos, sqrt, atan2
from database_utils import save_database, load_database

body_colors = {
    '0': '000000',    # Solar system barycenter
    '10': 'FFFFFF',   # Sun
    '1': '726658',    # Mercury barycenter
    '199': '726658',  # Mercury
    '2': 'EFECDD',    # Venus barycenter
    '299': 'EFECDD',  # Venus
    '3': 'A49FB3',    # Earth barycenter
    '399': 'A49FB3',  # Earth
    '4': '896545',    # Mars barycenter
    '499': '896545',  # Mars
    '5': 'C3BEAB',    # Jupiter barycenter
    '599': 'C3BEAB',  # Jupiter
    '6': 'C9B38E',    # Saturn barycenter
    '699': 'C9B38E',  # Saturn
    '7': 'AED5DA',    # Uranus barycenter
    '799': 'AED5DA',  # Uranus
    '8': '91AFBA',    # Neptune barycenter
    '899': '91AFBA',  # Neptune
    '9': 'C09F82',    # Pluto barycenter
    '999': 'C09F82',  # Pluto
}

materials = {
    '199': 'mercury',  # Mercury
    '299': 'venus',  # Venus
    '399': 'earth',  # Earth
    '499': 'mars',  # Mars
    '599': 'jupiter',  # Jupiter
    '699': 'saturn',  # Saturn
    '799': 'uranus',  # Uranus
    '899': 'neptune',  # Neptune
    '999': 'pluto',  # Pluto
}


def colorize(database):

    count = 0
    for db_name, db in database.items():
        if db_name in ['state_vectors', 'osc_elements']:
            continue

        for body_id, body in db.items():
            color = None
            try:
                color = body_colors[body_id]
                
                if 'material_params' not in body:
                    body['material_params'] = {}
                
                body['material_params']['diffuse_color'] = color
            except KeyError:
                pass

            mat = None
            try: 
                mat = materials[body_id]
                body['material'] = mat
            except KeyError:
                pass

            if color is not None or mat is not None:
                count += 1

    return count


def run(database):
    print("Adding material info to database bodies...")
    count = colorize(database)
    print(f"Added material info to {count} database bodies")


if __name__ == "__main__":
    database = load_database()

    run(database)

    save_database(database)
