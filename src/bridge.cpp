#include "duckdb.hpp"
#include "duckdb/main/capi/capi_internal.hpp"
#include "duckdb/common/assert.hpp"
#include "duckdb/function/scalar_function.hpp"
#include "duckdb/function/table_function.hpp"
#include "duckdb/parser/parsed_data/create_table_function_info.hpp"
#include "duckdb/parser/parsed_data/create_scalar_function_info.hpp"
#include "duckdb/planner/expression/bound_function_expression.hpp"
#include "duckdb/execution/expression_executor.hpp"
#include "duckdb/common/helper.hpp"
#include "bridge.hpp"

#include <iostream>
#include <memory>

using duckdb::Connection;
using duckdb::DuckDB;
using duckdb::Value;

namespace bridge {
    using namespace duckdb;
}

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

void jfr_scan_create_view(duckdb_client_context context, const char *filename, const char *tablename) {
    auto ctx = (duckdb::ClientContext *) context;
    auto conn = duckdb::Connection(ctx->db->GetDatabase(*ctx));
    conn.TableFunction("jfr_scan", {Value(filename), Value(tablename)})
            ->CreateView(tablename, true, false);
}

duckdb_file_handle duckdb_open_file(duckdb_client_context context, const char *path, uint8_t flags) {
    auto &fs = ((duckdb::ClientContext *) context)->db->GetFileSystem();
    auto handle = fs.OpenFile(path, flags);
    return handle.release();
}

int64_t duckdb_file_get_size(duckdb_file_handle handle) {
    return ((duckdb::FileHandle *) handle)->GetFileSize();
}

int64_t duckdb_file_read(duckdb_file_handle handle, void *buffer, int64_t nr_bytes) {
    return ((duckdb::FileHandle *) handle)->Read(buffer, nr_bytes);
}

void duckdb_file_seek(duckdb_file_handle handle, idx_t pos) {
    ((duckdb::FileHandle *) handle)->Seek(pos);
}

void duckdb_file_close(duckdb_file_handle handle) {
    delete (duckdb::FileHandle *) handle;
}

duckdb_logical_type duckdb_create_struct_type(
        idx_t n_pairs,
        const char **names,
        const duckdb_logical_type *types) {
    auto *stype = new duckdb::LogicalType;
    *stype = duckdb::LogicalType::STRUCT(getVector(n_pairs, names, types));
    return reinterpret_cast<duckdb_logical_type>(stype);
}

string_piece duckdb_get_string(duckdb_unified_vector_format vector, idx_t index) {
    auto fmt = reinterpret_cast<duckdb::UnifiedVectorFormat *>(vector);
    auto data = duckdb::UnifiedVectorFormat::GetData<duckdb::string_t>(*fmt);
    auto idx = fmt->sel->get_index(index);

    return string_piece{data[idx].GetData(), data[idx].GetSize()};
}

duckdb_unified_vector_format duckdb_unified_data_chunk_get_vector(duckdb_unified_data_chunk chunk, idx_t column) {
    auto dchunk = reinterpret_cast<duckdb::UnifiedVectorFormat *>(chunk);
    return reinterpret_cast<duckdb_unified_vector_format>(&dchunk[column]);
}

bool duckdb_unified_vector_validity_row_is_valid(duckdb_unified_vector_format vector, idx_t row) {
    auto fmt = reinterpret_cast<duckdb::UnifiedVectorFormat *>(vector);
    auto idx = fmt->sel->get_index(row);
    return fmt->validity.RowIsValid(idx);
}

}
