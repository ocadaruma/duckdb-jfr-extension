#ifndef DUCKDB_BUILD_LOADABLE_EXTENSION
#define DUCKDB_BUILD_LOADABLE_EXTENSION
#endif
#include "duckdb.h"

extern "C" {
//DUCKDB_EXTENSION_API const char* jfr_version();
//DUCKDB_EXTENSION_API void jfr_init(duckdb::DatabaseInstance &db);
void jfr_create_view(
        duckdb::Connection &connection,
        const char* filename,
        const char* tablename);

DUCKDB_EXTENSION_API duckdb_logical_type duckdb_create_struct_type(
        idx_t n_pairs,
        const char** names,
        const duckdb_logical_type* types);
}
