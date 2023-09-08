#ifndef DUCKDB_BUILD_LOADABLE_EXTENSION
#define DUCKDB_BUILD_LOADABLE_EXTENSION
#endif
#include "duckdb.h"

extern "C" {
typedef void *duckdb_file_handle;
typedef void *duckdb_client_context;
typedef void (*duckdb_table_function2_t)(duckdb_client_context ctx, duckdb_function_info info, duckdb_data_chunk output);

void jfr_scan_create_view(
        duckdb_client_context,
        const char* filename,
        const char* tablename);

//== A set of functions which are modified version of
//== original duckdb C APIs to support init/bind/function variants which accepts ClientContext
duckdb_table_function duckdb_create_table_function2();
void duckdb_table_function2_set_function(duckdb_table_function table_function, duckdb_table_function2_t function);
void duckdb_table_function2_set_bind(duckdb_table_function table_function, duckdb_table_function_bind_t bind);
void duckdb_table_function2_set_init(duckdb_table_function table_function, duckdb_table_function_init_t init);
duckdb_state duckdb_register_table_function2(duckdb_connection connection, duckdb_table_function function);
void *duckdb_function2_get_bind_data(duckdb_function_info info);
void *duckdb_function2_get_bind_data(duckdb_function_info info);
duckdb_logical_type duckdb_create_struct_type(
        idx_t n_pairs,
        const char** names,
        const duckdb_logical_type* types);

//== C APIs for file systems
duckdb_file_handle duckdb_open_file(duckdb_client_context context, const char *path, uint8_t flags);
int64_t duckdb_file_get_size(duckdb_file_handle handle);
int64_t duckdb_file_read(duckdb_file_handle handle, void *buffer, int64_t nr_bytes);
void duckdb_file_seek(duckdb_file_handle handle, idx_t pos);
void duckdb_file_close(duckdb_file_handle handle);

}
