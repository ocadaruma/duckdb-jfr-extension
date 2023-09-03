/*
 * because we link twice (once to the rust library, and once to the duckdb library) we need a bridge to export the rust symbols
 * this is that bridge
 */

#include "duckdb.hpp"
#include "duckdb/main/capi/capi_internal.hpp"
#include "wrapper.hpp"

#include <memory>

using duckdb::Connection;
using duckdb::DuckDB;
using duckdb::DatabaseData;
using duckdb::DatabaseInstance;
using duckdb::Value;

static duckdb::child_list_t<duckdb::LogicalType> getVector(
        idx_t n_pairs,
        const char *const *names,
        duckdb_logical_type const *types) {
    duckdb::child_list_t<duckdb::LogicalType> members;
    for (idx_t i = 0; i < n_pairs; i++) {
        members.emplace_back(
            std::string(names[i]),
            *(duckdb::LogicalType *) types[i]);
    }
    return members;
}

extern "C" {

// Bridge functions
void jfr_create_view(Connection &connection, const char* filename, const char* tablename) {
    connection.TableFunction("jfr_scan", {Value(filename), Value(tablename)})
            ->CreateView(tablename, true, false);
}

DUCKDB_EXTENSION_API duckdb_logical_type duckdb_create_struct_type(
        idx_t n_pairs,
        const char** names,
        const duckdb_logical_type* types) {
    auto *stype = new duckdb::LogicalType;
    *stype = duckdb::LogicalType::STRUCT(getVector(n_pairs, names, types));
    return reinterpret_cast<duckdb_logical_type>(stype);
}
}

