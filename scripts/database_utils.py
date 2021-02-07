import os
import sys
import json
import copy
import re

# database_folder = "D:/Dropbox/Astronomy/database"
database_folder = "E:/Rust/system_viewer/public/database"
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


def save_database(database):
    """ Write database back to their individual files """
    def order(item):
        try:
            return float(item[0])
        except ValueError:
            return item[0]

    # Sort all databases
    for db_name in database.keys():
        database[db_name] = {k: v for k, v in sorted(database[db_name].items(), key=lambda item: order(item))}

    # Write database to files
    for filename in database:
        path = os.path.join(database_folder, filename + ".json")
        with open(path, "w") as f:
            json.dump(database[filename], f)
