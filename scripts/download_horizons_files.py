import telnetlib
import os

# reference: https://github.com/tpltnt/py-NASA-horizons/blob/master/source/NASAhorizons.py
# telnet doc: https://docs.python.org/2/library/telnetlib.html

folder_path = r"E:\Rust\system_viewer\scripts\horizons_statevectors_ssb_j2000"


def get_orbital_elements(body, reference):
    """
    Returns orbital elements full text for 'body' wrt center 'reference'.
    Both body and reference follow the Horizons conventions.
    Example for Io around Jupiter system barycenter: getDataFromBody('501', '@5')

    Expects the telnet interface to be at 'Horizons>'. Will leave the interface
    back at this position as well
    :param body: HORIZONS tag for the target body
    :param reference: HORIZONS tag for the body center
    :return: Big string with all the returned text
    """
    tn.write(body.encode("ascii") + b"\n")
    body_data = tn.read_until(b"<cr>: ")
    tn.write(b"e\n")
    tn.read_until(b"?] :")
    tn.write(b"e\n")
    tn.read_until(b"? ] :")
    tn.write(reference.encode("ascii") + b"\n")
    tn.read_until(b"body ] :")
    tn.write(b"eclip\n")
    tn.read_until(b"] :")
    tn.write(b"2000-01-01 12:00\n")
    tn.read_until(b"] :")
    tn.write(b"2000-01-01 12:01\n")
    tn.read_until(b"] :")
    tn.write(b"1d\n")
    tn.read_until(b"?] :")
    tn.write(b"y\n")
    results = tn.read_until(b"? : ")
    tn.write(b"n\n")

    return body_data + b'\n\n\n' + results


def prepare_for_state_vectors():
    """
    Do a dummy query for state vectors so that it saves our default settings
    """
    tn.write(b"399\n")
    tn.read_until(b"<cr>: ")
    tn.write(b"e\n")
    tn.read_until(b"?] :")
    tn.write(b"v\n")
    tn.read_until(b"Coordinate center [ <id>,coord,geo  ] :")
    tn.write(b"@0\n")
    tn.read_until(b"body ] :")
    tn.write(b"eclip\n")
    tn.read_until(b"] :")
    tn.write(b"2000-01-01 12:00\n")
    tn.read_until(b"] :")
    tn.write(b"2000-01-01 12:01\n")
    tn.read_until(b"] :")
    tn.write(b"1d\n")
    tn.read_until(b"Accept default output [ cr=(y), n, ?] :")
    tn.write(b"n\n")
    tn.read_until(b"Output reference frame [ ICRF, B1950] :")
    tn.write(b"ICRF\n")
    tn.read_until(b"Corrections [ 1=NONE, 2=LT, 3=LT+S ]  :")
    tn.write(b"1\n")
    tn.read_until(b"Output units [1=KM-S, 2=AU-D, 3=KM-D] :")
    tn.write(b"1\n")
    tn.read_until(b"Spreadsheet CSV format    [ YES, NO ] :")
    tn.write(b"YES\n")
    tn.read_until(b"Output delta-T (TDB-UT)   [ YES, NO ] :")
    tn.write(b"YES\n")
    tn.read_until(b"Select output table type  [ 1-6, ?  ] :")
    tn.write(b"2\n")
    tn.read_until(b"Select... [A]gain, [N]ew-case, [F]tp, [M]ail, [R]edisplay, ? :")
    tn.write(b"n\n")


def get_state_vectors(body):
    """
    Returns state vectors full text for 'body' wrt center '@0'.
    Both body and reference follow the Horizons conventions.

    Expects the telnet interface to be at 'Horizons>'. Will leave the interface
    back at this position as well
    :param body: HORIZONS tag for the target body
    :return: Big string with all the returned text
    """
    tn.write(body.encode("ascii") + b"\n")
    body_data = tn.read_until(b"<cr>: ")
    tn.write(b"e\n")
    tn.read_until(b"?] :")
    tn.write(b"v\n")
    tn.read_until(b"Use previous center  [ cr=(y), n, ? ] : ")
    tn.write(b"y\n")
    tn.read_until(b"Reference plane [eclip, frame, body ] : ")
    tn.write(b"eclip\n")
    tn.read_until(b"] : ")
    tn.write(b"2000-01-01 12:00\n")
    tn.read_until(b"] : ")
    tn.write(b"2000-01-31 12:01\n")
    tn.read_until(b"] : ")
    tn.write(b"1d\n")
    tn.read_until(b"Accept default output [ cr=(y), n, ?] : ")
    tn.write(b"y\n")
    results = tn.read_until(b"Select... [A]gain, [N]ew-case, [F]tp, [M]ail, [R]edisplay, ? :")
    tn.write(b"n\n")

    return body_data + b'\n\n\n' + results


def save_data(filename, data):
    full_path = os.path.join(folder_path, filename + '.txt')

    with open(full_path, "w") as text_file:
        print(data, file=text_file)


def download_body(body):
    data = get_state_vectors(body).decode('ascii').replace('\r\n', '\n').replace('\n\n', '\n')
    save_data(body, data)


tn = telnetlib.Telnet("ssd.jpl.nasa.gov", 6775)
tn.set_debuglevel(999)
tn.read_until(b'Horizons>')

# Prime the 'default settings'
prepare_for_state_vectors()

# Planets wrt their own system barycenters
download_body('10')
download_body('199')
download_body('299')
download_body('399')
download_body('499')
download_body('599')
download_body('699')
download_body('799')
download_body('899')
download_body('999')

# Other satellites
download_body('301')
download_body('401')
download_body('402')
download_body('901')
download_body('902')
download_body('903')
download_body('904')
download_body('905')

# Jovian satellites
for i in range(501, 560):
    download_body(str(i))
download_body('55060')
download_body('55061')
download_body('55062')
download_body('55064')
download_body('55065')
download_body('55066')
download_body('55068')
download_body('55070')
download_body('55071')
download_body('55074')

# Saturnian satellites
for i in range(601, 654):
    download_body(str(i))
download_body('65035')
download_body('65040')
download_body('65041')
download_body('65045')
download_body('65048')
download_body('65050')
download_body('65055')
download_body('65056')

# Uranian satellites
for i in range(701, 728):
    download_body(str(i))

# Neptunian satellites
for i in range(801, 815):
    download_body(str(i))
