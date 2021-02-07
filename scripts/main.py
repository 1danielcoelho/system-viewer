from database_utils import save_database, load_database
import estimate_asteroid_parameters
import generate_state_vectors
import parse_horizons_ephemeris
import parse_jpl_satellite_physical_params
import parse_small_body_db_results

if __name__ == "__main__":
    db = load_database()

    parse_horizons_ephemeris.run(db)

    parse_small_body_db_results.run(db)

    parse_jpl_satellite_physical_params.run(db)

    estimate_asteroid_parameters.run(db)

    generate_state_vectors.run(db)

    save_database(db)
