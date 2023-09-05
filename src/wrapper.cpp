/*
 * because we link twice (once to the rust library, and once to the duckdb library) we need a bridge to export the rust symbols
 * this is that bridge
 */

#include "duckdb.hpp"
#include "duckdb/main/capi/capi_internal.hpp"
#include "duckdb/function/table_function.hpp"
#include "duckdb/parser/parsed_data/create_table_function_info.hpp"
#include "wrapper.hpp"

#include <iostream>
#include <memory>

using duckdb::Connection;
using duckdb::DuckDB;
using duckdb::Value;

/// Taken from table_function-c.cpp

namespace duckdb2 {
    using namespace duckdb;

    struct CTableFunctionInfo : public TableFunctionInfo {
        ~CTableFunctionInfo() {
            if (extra_info && delete_callback) {
                delete_callback(extra_info);
            }
            extra_info = nullptr;
            delete_callback = nullptr;
        }

        duckdb_table_function_bind_t bind = nullptr;
        duckdb_table_function_init_t init = nullptr;
        duckdb_table_function_init_t local_init = nullptr;
        duckdb_table_function2_t function = nullptr;
        void *extra_info = nullptr;
        duckdb_delete_callback_t delete_callback = nullptr;
    };

    struct CTableBindData : public TableFunctionData {
        CTableBindData(CTableFunctionInfo &info) : info(info) {
        }

        ~CTableBindData() {
            if (bind_data && delete_callback) {
                delete_callback(bind_data);
            }
            bind_data = nullptr;
            delete_callback = nullptr;
        }

        CTableFunctionInfo &info;
        void *bind_data = nullptr;
        duckdb_delete_callback_t delete_callback = nullptr;
        unique_ptr <NodeStatistics> stats;
    };

    struct CTableInternalBindInfo {
        CTableInternalBindInfo(ClientContext &context, TableFunctionBindInput &input,
                               vector <LogicalType> &return_types,
                               vector <string> &names, CTableBindData &bind_data, CTableFunctionInfo &function_info)
                : context(context), input(input), return_types(return_types), names(names), bind_data(bind_data),
                  function_info(function_info), success(true) {
        }

        ClientContext &context;
        TableFunctionBindInput &input;
        vector <LogicalType> &return_types;
        vector <string> &names;
        CTableBindData &bind_data;
        CTableFunctionInfo &function_info;
        bool success;
        string error;
    };

    struct CTableInitData {
        ~CTableInitData() {
            if (init_data && delete_callback) {
                delete_callback(init_data);
            }
            init_data = nullptr;
            delete_callback = nullptr;
        }

        void *init_data = nullptr;
        duckdb_delete_callback_t delete_callback = nullptr;
        idx_t max_threads = 1;
    };

    struct CTableGlobalInitData : public GlobalTableFunctionState {
        CTableInitData init_data;

        idx_t MaxThreads() const override {
            return init_data.max_threads;
        }
    };

    struct CTableLocalInitData : public LocalTableFunctionState {
        CTableInitData init_data;
    };

    struct CTableInternalInitInfo {
        CTableInternalInitInfo(const CTableBindData &bind_data, CTableInitData &init_data,
                               const vector <column_t> &column_ids, optional_ptr <TableFilterSet> filters)
                : bind_data(bind_data), init_data(init_data), column_ids(column_ids), filters(filters), success(true) {
        }

        const CTableBindData &bind_data;
        CTableInitData &init_data;
        const vector <column_t> &column_ids;
        optional_ptr <TableFilterSet> filters;
        bool success;
        string error;
    };

    struct CTableInternalFunctionInfo {
        CTableInternalFunctionInfo(const CTableBindData &bind_data, CTableInitData &init_data,
                                   CTableInitData &local_data)
                : bind_data(bind_data), init_data(init_data), local_data(local_data), success(true) {
        }

        const CTableBindData &bind_data;
        CTableInitData &init_data;
        CTableInitData &local_data;
        bool success;
        string error;
    };

    unique_ptr<FunctionData> CTableFunctionBind(ClientContext &context, TableFunctionBindInput &input,
                                                vector<LogicalType> &return_types, vector<string> &names) {
        auto &info = input.info->Cast<CTableFunctionInfo>();
        D_ASSERT(info.bind && info.function && info.init);
        auto result = make_uniq<CTableBindData>(info);
        CTableInternalBindInfo bind_info(context, input, return_types, names, *result, info);
        info.bind(&bind_info);
        if (!bind_info.success) {
            throw Exception(bind_info.error);
        }

        return std::move(result);
    }

    unique_ptr<GlobalTableFunctionState> CTableFunctionInit(ClientContext &context, TableFunctionInitInput &data_p) {
        auto &bind_data = data_p.bind_data->Cast<CTableBindData>();
        auto result = make_uniq<CTableGlobalInitData>();

        CTableInternalInitInfo init_info(bind_data, result->init_data, data_p.column_ids, data_p.filters);
        bind_data.info.init(&init_info);
        if (!init_info.success) {
            throw Exception(init_info.error);
        }
        return std::move(result);
    }

    unique_ptr<LocalTableFunctionState> CTableFunctionLocalInit(ExecutionContext &context, TableFunctionInitInput &data_p,
                                                                GlobalTableFunctionState *gstate) {
        auto &bind_data = data_p.bind_data->Cast<CTableBindData>();
        auto result = make_uniq<CTableLocalInitData>();
        if (!bind_data.info.local_init) {
            return std::move(result);
        }

        CTableInternalInitInfo init_info(bind_data, result->init_data, data_p.column_ids, data_p.filters);
        bind_data.info.local_init(&init_info);
        if (!init_info.success) {
            throw Exception(init_info.error);
        }
        return std::move(result);
    }

    void CTableFunction(ClientContext &context, TableFunctionInput &data_p, DataChunk &output) {
        auto &fs = context.db->GetFileSystem();
        auto &bind_data = data_p.bind_data->Cast<CTableBindData>();
        auto &global_data = (CTableGlobalInitData &)*data_p.global_state;
        auto &local_data = (CTableLocalInitData &)*data_p.local_state;
        CTableInternalFunctionInfo function_info(bind_data, global_data.init_data, local_data.init_data);
        bind_data.info.function(&context, &function_info, reinterpret_cast<duckdb_data_chunk>(&output));
        if (!function_info.success) {
            throw Exception(function_info.error);
        }
    }

    unique_ptr<NodeStatistics> CTableFunctionCardinality(ClientContext &context, const FunctionData *bind_data_p) {
        auto &bind_data = bind_data_p->Cast<CTableBindData>();
        if (!bind_data.stats) {
            return nullptr;
        }
        return make_uniq<NodeStatistics>(*bind_data.stats);
    }
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

// Bridge functions
void jfr_create_view(Connection &connection, const char* filename, const char* tablename) {
    connection.TableFunction("jfr_scan", {Value(filename), Value(tablename)})
            ->CreateView(tablename, true, false);
}

duckdb_table_function duckdb_create_table_function2() {
    auto function = new duckdb2::TableFunction("", {}, duckdb2::CTableFunction, duckdb2::CTableFunctionBind,
                                              duckdb2::CTableFunctionInit, duckdb2::CTableFunctionLocalInit);
    function->function_info = duckdb::make_shared<duckdb2::CTableFunctionInfo>();
    function->cardinality = duckdb2::CTableFunctionCardinality;
    return function;
}

void duckdb_table_function2_set_function(
        duckdb_table_function table_function,
        duckdb_table_function2_t function) {
    if (!table_function || !function) {
        return;
    }
    auto tf = (duckdb::TableFunction *)table_function;
    auto info = (duckdb2::CTableFunctionInfo *)tf->function_info.get();
    info->function = function;
}

void duckdb_table_function2_set_bind(duckdb_table_function function, duckdb_table_function_bind_t bind) {
    if (!function || !bind) {
        return;
    }
    auto tf = (duckdb::TableFunction *)function;
    auto info = (duckdb2::CTableFunctionInfo *)tf->function_info.get();
    info->bind = bind;
}

void duckdb_table_function2_set_init(duckdb_table_function function, duckdb_table_function_init_t init) {
    if (!function || !init) {
        return;
    }
    auto tf = (duckdb::TableFunction *)function;
    auto info = (duckdb2::CTableFunctionInfo *)tf->function_info.get();
    info->init = init;
}

duckdb_state duckdb_register_table_function2(duckdb_connection connection, duckdb_table_function function) {
    if (!connection || !function) {
        return DuckDBError;
    }
    auto con = (duckdb::Connection *)connection;
    auto tf = (duckdb::TableFunction *)function;
    auto info = (duckdb2::CTableFunctionInfo *)tf->function_info.get();
    if (tf->name.empty() || !info->bind || !info->init || !info->function) {
        return DuckDBError;
    }
    con->context->RunFunctionInTransaction([&]() {
        auto &catalog = duckdb::Catalog::GetSystemCatalog(*con->context);
        duckdb2::CreateTableFunctionInfo tf_info(*tf);

        // create the function in the catalog
        catalog.CreateTableFunction(*con->context, tf_info);
    });
    return DuckDBSuccess;
}

void *duckdb_function2_get_bind_data(duckdb_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (duckdb2::CTableInternalFunctionInfo *)info;
    return function_info->bind_data.bind_data;
}

void *duckdb_function2_get_init_data(duckdb_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (duckdb2::CTableInternalFunctionInfo *)info;
    return function_info->init_data.init_data;
}

duckdb::FileHandle* duckdb_open_file(duckdb::ClientContext & context, const char *path, uint8_t flags) {
//    duckdb::ClientContext &ctx = reinterpret_cast<duckdb::ClientContext &>(context);
    auto &fs = context.db->GetFileSystem();
    auto handle = fs.OpenFile(path, flags);
    auto ptr = handle.release();
    std::cout << "opened file: " << ptr << std::endl;
    return ptr;
}

int64_t duckdb_file_get_size(duckdb::FileHandle* handle) {
//    std::cout << "duckdb_file_get_size: " << &handle << std::endl;
    return handle->GetFileSize();
}

int64_t duckdb_file_read(duckdb::FileHandle* handle, void *buffer, int64_t nr_bytes) {
    return handle->Read(buffer, nr_bytes);
}

void duckdb_file_seek(duckdb::FileHandle* handle, idx_t pos) {
    handle->Seek(pos);
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

