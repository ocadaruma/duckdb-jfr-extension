/*
 * because we link twice (once to the rust library, and once to the duckdb library) we need a bridge to export the rust symbols
 * this is that bridge
 */

#include "duckdb.hpp"
#include "duckdb/main/capi/capi_internal.hpp"
#include "wrapper.hpp"

using duckdb::Connection;
using duckdb::DuckDB;
using duckdb::DatabaseData;
using duckdb::DatabaseInstance;
using duckdb::Value;

extern "C" {
const char* jfr_version_rust(void);
void jfr_init_rust(DatabaseInstance &db);

DUCKDB_EXTENSION_API const char* jfr_version() {
    return jfr_version_rust();
}

DUCKDB_EXTENSION_API void jfr_init(DatabaseInstance &db) {
//    auto wrapper = new DatabaseData();
//    wrapper->database = duckdb::make_uniq<DuckDB>(db);
    jfr_init_rust(db);
}

// Bridge functions
void jfr_create_view(Connection &connection, const char* filename, const char* tablename) {
    connection.TableFunction("jfr_scan", {Value(filename), Value(tablename)})
            ->CreateView(tablename, true, false);
}
}

