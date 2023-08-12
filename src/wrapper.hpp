#ifndef DUCKDB_BUILD_LOADABLE_EXTENSION
#define DUCKDB_BUILD_LOADABLE_EXTENSION
#endif
#include "duckdb.h"

extern "C" {
DUCKDB_EXTENSION_API const char* jfr_version();
DUCKDB_EXTENSION_API void jfr_init(duckdb::DatabaseInstance &db);
void jfr_create_view(duckdb::Connection &connection, const char* tablename);
}
