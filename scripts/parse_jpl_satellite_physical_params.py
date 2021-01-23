# This parses https://ssd.jpl.nasa.gov/?sat_phys_par and injects the data into our database
# If it finds a conflicting value it will not overwrite it, because we want as much data as possible to
# match Horizons (which is our primary source) given that we will compare state vectors using it

from math import nan
from database_utils import load_database, save_database, get_body_by_name
from constants import *

values = [
    # Name              # GM (km3/s2)   Radius (km) Density(g/cm3) Magnitude    Albedo
    ["Moon",	        4902.801,		1737.5,	    3.344,      "-12.74",	    0.12],
    ["Phobos",	        0.0007112,		11.1,	    1.872,      "11.4",	        0.071],
    ["Deimos",	        0.0000985,		6.2,	    1.471,      "12.45",	    0.068],
    ["Io",	            5959.916,		1821.6,	    3.528,      "5.02",	        0.63],
    ["Europa",	        3202.739,		1560.8,	    3.013,      "5.29",	        0.67],
    ["Ganymede",	    9887.834,		2631.2,	    1.942,      "4.61",	        0.43],
    ["Callisto",	    7179.289,		2410.3,	    1.834,      "5.65",	        0.17],
    ["Amalthea",	    0.138,		    83.45,	    0.849,      "14.1",	        0.090],
    ["Himalia",	        0.45, 	        85,	        2.6,        "14.2R",		0.04],
    ["Elara",	        0.058, 	        43,	        2.6,        "16.0R",		0.04],
    ["Pasiphae",	    0.020, 	        30,	        2.6,        "16.8R",		0.04],
    ["Sinope",	        0.0050, 	    19,	        2.6,        "18.2R",		0.04],
    ["Lysithea",	    0.0042, 	    18,	        2.6,        "18.1R",		0.04],
    ["Carme",	        0.0088, 	    23,	        2.6,        "18.1R",		0.04],
    ["Ananke",	        0.0020, 	    14,	        2.6,        "19.1R",		0.04],
    ["Leda",	        0.00073, 	    10,	        2.6,        "19.2R",		0.04],
    ["Thebe",	        0.10, 	        49.3,	    3.0,        "16.0",	        0.047],
    ["Adrastea",	    0.0005, 	    8.2,	    3.0,        "18.7",	        0.1],
    ["Metis",	        0.008, 	        21.5,	    3.0,        "17.5",	        0.061],
    ["Callirrhoe",	    0.000058, 	    4.3,	    2.6,        "20.8R",		0.04],
    ["Themisto",	    0.000046, 	    4.0,	    2.6,        "21.0R",		0.04],
    ["Megaclite",	    0.000014, 	    2.7,	    2.6,        "21.7R",		0.04],
    ["Taygete",	        0.000011, 	    2.5,	    2.6,        "21.9R",		0.04],
    ["Chaldene",	    0.000005, 	    1.9,	    2.6,        "22.5R",		0.04],
    ["Harpalyke",	    0.000008, 	    2.2,	    2.6,        "22.2R",		0.04],
    ["Kalyke",	        0.000013, 	    2.6,	    2.6,        "21.8R",		0.04],
    ["Iocaste",	        0.000013, 	    2.6,	    2.6,        "21.8R",		0.04],
    ["Erinome",	        0.000003, 	    1.6,	    2.6,        "22.8R",		0.04],
    ["Isonoe",	        0.000005, 	    1.9,	    2.6,        "22.5R",		0.04],
    ["Praxidike",	    0.000029, 	    3.4,	    2.6,        "21.2R",		0.04],
    ["Autonoe",	        0.000006, 	    2.0,	    2.6,        "22.0R",		0.04],
    ["Thyone",	        0.000006, 	    2.0,	    2.6,        "22.3R",		0.04],
    ["Hermippe",	    0.000006, 	    2.0,	    2.6,        "22.1R",		0.04],
    ["Aitne",	        0.000003, 	    1.5,	    2.6,        "22.7R",		0.04],
    ["Eurydome",	    0.000003, 	    1.5,	    2.6,        "22.7R",		0.04],
    ["Euanthe",	        0.000003, 	    1.5,	    2.6,        "22.8R",		0.04],
    ["Euporie",	        0.000001, 	    1.0,	    2.6,        "23.1R",		0.04],
    ["Orthosie",	    0.000001, 	    1.0,	    2.6,        "23.1R",		0.04],
    ["Sponde",	        0.000001, 	    1.0,	    2.6,        "23.0R",		0.04],
    ["Kale",	        0.000001, 	    1.0,	    2.6,        "23.0R",		0.04],
    ["Pasithee",	    0.000001, 	    1.0,	    2.6,        "23.2R",		0.04],
    ["Hegemone",	    0.000003, 	    1.5,	    2.6,        "22.8R",		0.04],
    ["Mneme",	        0.000001, 	    1.0,	    2.6,        "23.3R",		0.04],
    ["Aoede",	        0.000006, 	    2.0,	    2.6,        "22.5R",		0.04],
    ["Thelxinoe",	    0.000001, 	    1.0,	    2.6,        "23.5R",		0.04],
    ["Arche",	        0.000003, 	    1.5,	    2.6,        "22.8R",		0.04],
    ["Kallichore",	    0.000001, 	    1.0,	    2.6,        "23.7R",		0.04],
    ["Helike",	        0.000006, 	    2.0,	    2.6,        "22.6R",		0.04],
    ["Carpo",	        0.000003, 	    1.5,	    2.6,        "23.0R",		0.04],
    ["Eukelade",	    0.000006, 	    2.0,	    2.6,        "22.6R",		0.04],
    ["Cyllene",	        0.000001, 	    1.0,	    2.6,        "23.2R",		0.04],
    ["Kore",	        0.000001, 	    1.0,	    2.6,        "23.6R",		0.04],
    ["Herse",	        0.000001, 	    1.0,	    2.6,        "23.4R",		0.04],
    ["2000J11",	        0.000001, 	    1.0,	    2.6,        "22.4R",		0.04],
    ["2003J2",	        0.000001, 	    1.0,	    2.6,        "23.2R",		0.04],
    ["2003J3",	        0.000001, 	    1.0,	    2.6,        "23.4R",		0.04],
    ["2003J4",	        0.000001, 	    1.0,	    2.6,        "23.0R",		0.04],
    ["2003J5",	        0.000006, 	    2.0,	    2.6,        "22.4R",		0.04],
    ["2003J9",	        0.0000001, 	    0.5,	    2.6,        "23.7R",		0.04],
    ["2003J10",	        0.000001, 	    1.0,	    2.6,        "23.6R",		0.04],
    ["2003J12",	        0.0000001, 	    0.5,	    2.6,        "23.9R",		0.04],
    ["2003J15",	        0.000001, 	    1.0,	    2.6,        "23.5R",		0.04],
    ["2003J16",	        0.000001, 	    1.0,	    2.6,        "23.3R",		0.04],
    ["2003J18",	        0.000001, 	    1.0,	    2.6,        "23.4R",		0.04],
    ["2003J19",	        0.000001, 	    1.0,	    2.6,        "23.7R",		0.04],
    ["2003J23",	        0.000001, 	    1.0,	    2.6,        "23.6R",		0.04],
    ["2010J1",	        0.000001, 	    1.0,	    2.6,        "23.2r",		0.04],
    ["2010J2",	        0.000001, 	    1.0,	    2.6,        "24.0r",		0.04],
    ["2011J1",	        0.000001, 	    1.0,	    2.6,        "23.7R",		0.04],
    ["2011J2",	        0.000001, 	    1.0,	    2.6,        "23.5R",		0.04],
    ["Mimas",	        2.5026,		    198.20,	    1.150,      "12.8",	        0.962],
    ["Enceladus",	    7.2027,		    252.10,	    1.608,      "11.8",	        1.375],
    ["Tethys",	        41.2067,		533.00,	    0.973,      "10.2",	        1.229],
    ["Dione",	        73.1146,		561.70,	    1.476,      "10.4",	        0.998],
    ["Rhea",	        153.9426,		764.30,	    1.233,      "9.6",          0.949],
    ["Titan",	        8978.1382,		2574.73,	1.882,      "8.4",          0.2],
    ["Hyperion",	    0.3727,		    135.00,	    0.544,      "14.4",	        0.3],
    ["Iapetus",	        120.5038,		735.60,	    1.083,      "11",           0.6],
    ["Phoebe",	        0.5532,		    106.50,	    1.638,      "16.4",	        0.081],
    ["Janus",	        0.1263,		    89.5,	    0.630,      "14.4",	        0.71],
    ["Epimetheus",	    0.0351,		    58.1,	    0.640,      "15.6",	        0.73],
    ["Helene",	        0.00076, 	    17.6,	    0.5,        "18.4",	        1.67],
    ["Telesto",	        0.00027, 	    12.4,	    0.5,        "18.5",	        1.0],
    ["Calypso",	        0.00017, 	    10.7,	    0.5,        "18.7",	        1.34],
    ["Atlas",	        0.00044,		15.1,	    0.460,      "19.0",	        0.4],
    ["Prometheus",	    0.01074,		43.1,	    0.480,      "15.8",	        0.6],
    ["Pandora",	        0.00924,		40.7,	    0.490,      "16.4",	        0.5],
    ["Pan",	            0.00033,		14.1,	    0.420,      "19.4",	        0.5],
    ["Methone",	        0.0000006, 	    1.6,	    0.5,        "?",            nan],
    ["Pallene",	        0.0000022, 	    2.5,	    0.5,        "?",            nan],
    ["Polydeuces",	    0.0000003, 	    1.3,	    0.5,        "?",            nan],
    ["Daphnis",	        0.0000052,		3.8,	    0.340,      "?",            nan],
    ["Anthe",	        0.0000001, 	    0.9,	    0.5,        "?",            nan],
    ["Aegaeon",	        0.000000004, 	0.3,	    0.5,        "?",            nan],
    ["Ymir",	        0.00033, 	    9,	        2.3,        "21.9R",		0.06],
    ["Paaliaq",	        0.00055, 	    11.0,	    2.3,        "21.1R",		0.06],
    ["Tarvos",	        0.00018, 	    7.5,	    2.3,        "22.7R",		0.06],
    ["Ijiraq",	        0.000080, 	    6,	        2.3,        "22.6R",		0.06],
    ["Suttungr",	    0.000014, 	    3.5,	    2.3,        "23.9R",		0.06],
    ["Kiviuq",	        0.00022, 	    8,	        2.3,        "22.1R",		0.06],
    ["Mundilfari",	    0.000014, 	    3.5,	    2.3,        "23.8R",		0.06],
    ["Albiorix",	    0.0014, 	    16,	        2.3,        "20.5R",		0.06],
    ["Skathi",	        0.000021, 	    4,	        2.3,        "23.6R",		0.06],
    ["Erriapus",	    0.000051, 	    5,	        2.3,        "23.4R",		0.06],
    ["Siarnaq",	        0.0026, 	    20,	        2.3,        "19.9R",		0.06],
    ["Thrymr",	        0.000014, 	    3.5,	    2.3,        "23.9R",		0.06],
    ["Narvi",	        0.000023, 	    3.5,	    2.3,        "23.8R",		0.04],
    ["Aegir",	        0.000000, 	    3.0,	    2.3,        "24.4R",		0.04],
    ["Bebhionn",	    0.000000, 	    3.0,	    2.3,        "24.1R",		0.04],
    ["Bergelmir",	    0.000000, 	    3.0,	    2.3,        "24.2R",		0.04],
    ["Bestla",	        0.000000, 	    3.5,	    2.3,        "23.8R",		0.04],
    ["Farbauti",	    0.000000, 	    2.5,	    2.3,        "24.7R",		0.04],
    ["Fenrir",	        0.000000, 	    2.0,	    2.3,        "25.0R",		0.04],
    ["Fornjot",	        0.000000, 	    3.0,	    2.3,        "24.6R",		0.04],
    ["Hati",	        0.000000, 	    3.0,	    2.3,        "24.4R",		0.04],
    ["Hyrrokkin",	    0.000000, 	    3.0,	    2.3,        "23.5R",		0.04],
    ["Kari",	        0.000000, 	    3.0,	    2.3,        "23.9R",		0.04],
    ["Loge",	        0.000000, 	    3.0,	    2.3,        "24.6R",		0.04],
    ["Skoll",	        0.000000, 	    3.0,	    2.3,        "24.5R",		0.04],
    ["Surtur",	        0.000000, 	    3.0,	    2.3,        "24.8R",		0.04],
    ["Jarnsaxa",	    0.000000, 	    3.0,	    2.3,        "24.7R",		0.04],
    ["Greip",	        0.000000, 	    3.0,	    2.3,        "24.4R",		0.04],
    ["Tarqeq",	        0.000000, 	    3.0,	    2.3,        "23.9R",		0.04],
    ["2004S7",	        0.000000, 	    3.0,	    2.3,        "24.5R",		0.04],
    ["2004S12",	        0.000000, 	    2.5,	    2.3,        "24.8R",		0.04],
    ["2004S13",	        0.000000, 	    3.0,	    2.3,        "24.5R",		0.04],
    ["2004S17",	        0.000000, 	    2.0,	    2.3,        "25.2R",		0.04],
    ["2006S1",	        0.000000, 	    3.0,	    2.3,        "24.6R",		0.04],
    ["2006S3",	        0.000000, 	    2.5,	    2.3,        "24.6R",		0.04],
    ["2007S2",	        0.000000, 	    3.0,	    2.3,        "24.4R",		0.04],
    ["2007S3",	        0.000000, 	    2.0,	    2.3,        "24.9R",		0.04],
    ["Ariel",	        86.4,		    578.9,	    1.592,      "13.70",	    0.39],
    ["Umbriel",	        81.5,		    584.7,	    1.459,      "14.47",	    0.21],
    ["Titania",	        228.2,		    788.9,	    1.662,      "13.49",	    0.27],
    ["Oberon",	        192.4,		    761.4,	    1.559,      "13.70",	    0.23],
    ["Miranda",	        4.4,		    235.8,	    1.214,      "15.79",	    0.32],
    ["Cordelia",	    0.0030, 	    20.1,	    1.3,        "23.62",	    0.07],
    ["Ophelia",	        0.0036, 	    21.4,	    1.3,        "23.26",	    0.07],
    ["Bianca",	        0.0062, 	    27,	        1.3,        "22.52",	    0.065],
    ["Cressida",	    0.0229, 	    41,	        1.3,        "21.58",	    0.069],
    ["Desdemona",	    0.0119, 	    35,	        1.3,        "21.99",	    0.084],
    ["Juliet",	        0.0372, 	    53,	        1.3,        "21.12",	    0.075],
    ["Portia",	        0.1122, 	    70,	        1.3,        "20.42",	    0.069],
    ["Rosalind",	    0.0170, 	    36,	        1.3,        "21.79",	    0.072],
    ["Belinda",	        0.0238, 	    45,	        1.3,        "21.47",	    0.067],
    ["Puck",	        0.1931, 	    81,	        1.3,        "19.75",	    0.104],
    ["Caliban",	        0.020, 	        36,	        1.5,        "22.4R",		0.04],
    ["Sycorax",	        0.18, 	        75,	        1.5,        "20.8R",		0.04],
    ["Prospero",	    0.0066, 	    25,	        1.5,        "23.2R",		0.04],
    ["Setebos",	        0.0058, 	    24,	        1.5,        "23.3R",		0.04],
    ["Stephano",	    0.0017, 	    16,	        1.5,        "24.1R",		0.04],
    ["Trinculo",	    0.00031, 	    9,	        1.5,        "25.4R",		0.04],
    ["Francisco",	    0.00056, 	    11,	        1.5,        "25.0R",		0.04],
    ["Margaret",	    0.00042, 	    10,	        1.5,        "25.2R",		0.04],
    ["Ferdinand",	    0.00042, 	    10,	        1.5,        "25.1R",		0.04],
    ["Perdita",	        0.0012, 	    13,	        1.3,        "23.6V",		0.070],
    ["Mab",	            0.0006, 	    12,	        1.3,        "24.6V",		0.103],
    ["Cupid",	        0.0002, 	    9,	        1.3,        "25.8V",		0.070],
    ["Triton",	        1427.6,		    1353.4,	    2.059,      "13.54",	    0.719],
    ["Nereid",	        2.06, 	        170.0,		1.5,        "19.2R",		0.155],
    ["Naiad",	        0.013, 	        33.0,		1.3,        "23.91",	    0.072],
    ["Thalassa",	    0.025, 	        41.0,		1.3,        "23.32",	    0.091],
    ["Despina",	        0.14, 	        75.0,		1.3,        "22.00",	    0.090],
    ["Galatea",	        0.25, 	        88.0,		1.3,        "21.85",	    0.079],
    ["Larissa",	        0.33, 	        97.0,		1.3,        "21.49",	    0.091],
    ["Proteus",	        3.36, 	        210.0,		1.3,        "19.75",	    0.096],
    ["Halimede",	    0.012, 	        31.0,	    1.5,        "24.5R",		0.04],
    ["Psamathe",	    0.0033, 	    20.0,	    1.5,        "25.5R",		0.04],
    ["Sao",	            0.0045, 	    22.0,	    1.5,        "25.5R",		0.04],
    ["Laomedeia",	    0.0039, 	    21.0,	    1.5,        "25.5R",		0.04],
    ["Neso",	        0.011, 	        30.0,	    1.5,        "24.6R",		0.04],
    ["2004N1",	        0.0003, 	    9.0,	    1.3,        "26.5V",		0.10],
    ["Charon",	        102.3,		    603.6,	    1.664,      "17.26",	    0.372],
    ["Nix",	            0.0013,		    23.0,	    2.1,        "23.4V",		0.35],
    ["Hydra",	        0.0065,		    30.5,	    0.8,        "22.9V",		0.35],
    ["Kerberos",	    0.0011,		    14.0,	    1.4,        "26.1V",		0.35],
    ["Styx",	        0.0000,		    10.0,	    nan,	    "27.0V",		0.35],
]

def add_jpl_data(database):
    for name, mass, radius, density, magnitude, albedo in values:
        if mass == 0:
            continue
        mass_kg = mass / G

        body = get_body_by_name(database, name)
        if body is None:
            print("Failed to find a body for entry '" + str(name) + "'")
            continue

        try:
            body_mass = body['mass']
            divergence_pct = 100.0 * (abs(body_mass - mass_kg) / body_mass)
            if divergence_pct > 20:
                print(f'Found divergent mass for body {body["name"]}: HORIZONS value: "{body_mass:E} kg"; JPL value: "{mass_kg:E} kg" (divergence of {divergence_pct:.2f} %)')

        except KeyError:
            print('Assigning body %s the mass %s because it didn\'t have one' % (body['name'], mass_kg))
            body['mass'] = mass_kg



if __name__ == "__main__":
    database = load_database()

    add_jpl_data(database)

    save_database(database)
