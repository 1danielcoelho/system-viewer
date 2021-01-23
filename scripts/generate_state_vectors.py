from constants import *
from math import sin, cos, sqrt, atan2
from database_utils import save_database, load_database

newton_raphson_max_iter = 30
newton_raphson_delta = 0.0000000001


def elements_to_state_vectors(elements, t):
    """ Convert orbital elements to state vectors in Mm and Mm/s 

    Elements:
    {
      "ref_id": "0", // reference body id
      "epoch": 2455400.0, // JDN
      "a": 2, // Semi-major axis, Mm
      "e": 2, // Eccentricity, abs
      "i": 2, // Inclination, rad
      "O": 2, // Longitude of ascending node, rad
      "w": 2, // Argument of periapsis, rad
      "M": 2, // Mean anomaly, rad
      "p": 2, // Sidereal orbital period, days (86400s)
    },

    t: JDN instant for the state vectors

    Sources:
    https://space.stackexchange.com/questions/19322/converting-orbital-elements-to-cartesian-state-vectors
    https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
    """
    ref_id = elements['ref_id']
    assert(ref_id == '10')  # Only sun body center for now, as we have to compound orbits otherwise

    a = elements['a']
    e = elements['e']
    inc = elements['i']
    O = elements['O']
    w = elements['w']
    M0 = elements['M']
    p = elements['p']
    epoch = elements['epoch']

    while t < epoch:
        t += p

    n = 2.0 * PI / p
    u = n * n * (a ** 3.0)
    M = M0 + n * (t - epoch)

    E = M
    error = E - e * sin(E) - M
    for _ in range(newton_raphson_max_iter):
        if abs(error) <= newton_raphson_delta:
            break

        E = E - error / (1.0 - e * cos(E))
        error = E - e * sin(E) - M

    if error > newton_raphson_delta:
        print(f"Failed to converge for elements {str(elements)} at time {str(t)}")

    v = 2.0 * atan2(sqrt(1.0 + e) * sin(E / 2.0), sqrt(1.0 - e) * cos(E / 2.0))

    dist = a * (1.0 - e * cos(E))

    pos_x = dist * cos(v)
    pos_y = dist * sin(v)

    scalar = sqrt(u * a) / dist
    vel_x = scalar * (-sin(E))
    vel_y = scalar * sqrt(1.0 - e * e) * cos(E)

    cosw = cos(w)
    sinw = sin(w)
    coso = cos(O)
    sino = sin(O)
    cosi = cos(inc)
    sini = sin(inc)

    f_pos_x = pos_x * (cosw * coso - sinw * cosi * sino) - pos_y * (sinw * coso + cosw * cosi * sino)
    f_pos_y = pos_x * (cosw * sino + sinw * cosi * coso) + pos_y * (cosw * cosi * coso - sinw * sino)
    f_pos_z = pos_x * (sinw * sini) + pos_y * (cosw * sini)

    f_vel_x = vel_x * (cosw * coso - sinw * cosi * sino) - vel_y * (sinw * coso + cosw * cosi * sino)
    f_vel_y = vel_x * (cosw * sino + sinw * cosi * coso) + vel_y * (cosw * cosi * coso - sinw * sino)
    f_vel_z = vel_x * (sinw * sini) + vel_y * (cosw * sini)

    # TODO: Our osculating elements are wrt the sun, but our state vectors wrt the solar system barycenter,
    # and the sun is moving wrt that barycenter

    return [
        J2000,
        f_pos_x,
        f_pos_y,
        f_pos_z,
        f_vel_x / 86400,
        f_vel_y / 86400,
        f_vel_z / 86400
    ]


def helio_to_ssb(vec):
    sun_ssb_state_vectors = [-1.067598502264559E+03, -4.182343932742174E+02, 3.083761810502339E+01, 9.312570119052345E-06, -1.282474958274199E-05, -1.633335103087856E-07]
    assert(vec[0] == J2000)

    return [
        vec[0],
        vec[1] + sun_ssb_state_vectors[0],
        vec[2] + sun_ssb_state_vectors[1],
        vec[3] + sun_ssb_state_vectors[2],
        vec[4] + sun_ssb_state_vectors[3],
        vec[5] + sun_ssb_state_vectors[4],
        vec[6] + sun_ssb_state_vectors[5],
    ]


def test_elements_to_state_vectors():
    fake_osc = {}
    fake_osc['epoch'] = J2000
    fake_osc['ref_id'] = '10'
    fake_osc['e'] = 6.755786250503024E-03
    fake_osc['i'] = 3.394589648659516E+00 * deg_to_rad
    fake_osc['O'] = 7.667837463924961E+01 * deg_to_rad
    fake_osc['w'] = 5.518596653686583E+01 * deg_to_rad
    fake_osc['M'] = 5.011477187351476E+01 * deg_to_rad
    fake_osc['a'] = 7.233269274790103E-01 * au_to_Mm
    fake_osc['p'] = 2.246983300739057E+02
    print(fake_osc)

    fake_vec = [
        J2000,
        -1.074564940489116E+05,  # Mm
        -4.885015029930510E+03,
        6.135634314000621E+03,
        1.381906047920155E-03,  # Mm / s
        -3.514029517606325E-02,
        -5.600423209496981E-04,
    ]
    print("fake", fake_vec)

    calc_vec = elements_to_state_vectors(fake_osc, J2000)
    print("calc", calc_vec)

    database = load_database()
    merc_osc = database['major_bodies']['1']['osc_elements'][0]
    merc_vec = database['major_bodies']['199']['state_vectors'][0]

    print(merc_osc)
    print(merc_vec)

    calc_vec = elements_to_state_vectors(merc_osc, J2000)
    print(calc_vec)

    off_vec = helio_to_ssb(calc_vec)
    print(off_vec)


def ensure_j2000_state_vector(database):
    for db in database.values():
        for body_id, body in db.items():
            if 'osc_elements' not in body:
                continue

            # Don't do anything if the body already has a state vector for J2000
            try:
                has_j2000_vec = False
                for vec in body['state_vectors']:
                    if vec[0] == J2000:
                        has_j2000_vec = True
                        break

                if has_j2000_vec:
                    continue
            except KeyError:
                pass

            # Find osculating elements closest to J2000
            elements = {}
            for el in body['osc_elements']:
                if el['ref_id'] != '10':
                    continue

                epoch = el['epoch']
                if len(elements) == 0 or abs(epoch - J2000) < abs(elements['epoch'] - J2000):
                    elements = el

            # Skip if we can't use any elements (e.g. satellites)
            if len(elements) == 0:
                continue

            new_vec = elements_to_state_vectors(elements, J2000)
            new_vec = helio_to_ssb(new_vec)
            print(f'Computed state vectors for body {body_id} ("{body["name"]}"): {new_vec}')

            if 'state_vectors' not in body:
                body['state_vectors'] = []
            body['state_vectors'].append(new_vec)


if __name__ == "__main__":
    database = load_database()

    ensure_j2000_state_vector(database)

    save_database(database)
