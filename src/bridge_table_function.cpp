#include "bridge.hpp"
#include "duckdb.hpp"
#include "duckdb/parser/parsed_data/create_table_function_info.hpp"

/// Taken from table_function-c.cpp

namespace bridge {
    using namespace duckdb;

    struct CTableFunctionView {
        string function_name;
        string name;
        vector<Value> parameters;
    };

    struct CTableFunctionInfo : public TableFunctionInfo {
        ~CTableFunctionInfo() {
            if (extra_info && delete_callback) {
                delete_callback(extra_info);
            }
            extra_info = nullptr;
            delete_callback = nullptr;
        }

        duckdb_table_function2_bind_t bind = nullptr;
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
        info.bind(&context, &bind_info);
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

extern "C" {

duckdb_table_function duckdb_create_table_function2() {
    auto function = new duckdb::TableFunction("", {}, bridge::CTableFunction, bridge::CTableFunctionBind,
                                              bridge::CTableFunctionInit, bridge::CTableFunctionLocalInit);
    function->function_info = duckdb::make_shared<bridge::CTableFunctionInfo>();
    function->cardinality = bridge::CTableFunctionCardinality;
    return function;
}

void duckdb_table_function2_set_function(
        duckdb_table_function table_function,
        duckdb_table_function2_t function) {
    if (!table_function || !function) {
        return;
    }
    auto tf = (duckdb::TableFunction *) table_function;
    auto info = (bridge::CTableFunctionInfo *) tf->function_info.get();
    info->function = function;
}

void duckdb_table_function2_set_bind(duckdb_table_function function, duckdb_table_function2_bind_t bind) {
    if (!function || !bind) {
        return;
    }
    auto tf = (duckdb::TableFunction *) function;
    auto info = (bridge::CTableFunctionInfo *) tf->function_info.get();
    info->bind = bind;
}

void duckdb_table_function2_set_init(duckdb_table_function function, duckdb_table_function_init_t init) {
    if (!function || !init) {
        return;
    }
    auto tf = (duckdb::TableFunction *) function;
    auto info = (bridge::CTableFunctionInfo *) tf->function_info.get();
    info->init = init;
}

duckdb_state duckdb_register_table_function2(duckdb_connection connection, duckdb_table_function function) {
    if (!connection || !function) {
        return DuckDBError;
    }
    auto con = (duckdb::Connection *) connection;
    auto tf = (duckdb::TableFunction *) function;
    auto info = (bridge::CTableFunctionInfo *) tf->function_info.get();
    if (tf->name.empty() || !info->bind || !info->init || !info->function) {
        return DuckDBError;
    }
    con->context->RunFunctionInTransaction([&]() {
        auto &catalog = duckdb::Catalog::GetSystemCatalog(*con->context);
        duckdb::CreateTableFunctionInfo tf_info(*tf);

        // create the function in the catalog
        catalog.CreateTableFunction(*con->context, tf_info);
    });
    return DuckDBSuccess;
}

void *duckdb_function2_get_bind_data(duckdb_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (bridge::CTableInternalFunctionInfo *) info;
    return function_info->bind_data.bind_data;
}

void *duckdb_function2_get_init_data(duckdb_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (bridge::CTableInternalFunctionInfo *) info;
    return function_info->init_data.init_data;
}

duckdb_table_function_view duckdb_create_table_function_view() {
    return new bridge::CTableFunctionView();
}

void duckdb_table_function_view_set_function_name(duckdb_table_function_view view, const char *function_name) {
    if (!view || !function_name) {
        return;
    }
    auto tfv = (bridge::CTableFunctionView *) view;
    tfv->function_name = function_name;
}

void duckdb_table_function_view_set_name(duckdb_table_function_view view, const char *name) {
    if (!view || !name) {
        return;
    }
    auto tfv = (bridge::CTableFunctionView *) view;
    tfv->name = name;
}

void duckdb_table_function_view_add_parameter(duckdb_table_function_view view, duckdb_value value) {
    if (!view || !value) {
        return;
    }
    auto tfv = (bridge::CTableFunctionView *) view;
    auto val = (duckdb::Value *) value;
    tfv->parameters.push_back(*val);
}

duckdb_state duckdb_register_table_function_view(duckdb_client_context context, duckdb_table_function_view view) {
    if (!context || !view) {
        return DuckDBError;
    }
    auto ctx = (duckdb::ClientContext *) context;
    auto conn = duckdb::Connection(ctx->db->GetDatabase(*ctx));
    auto tfv = (bridge::CTableFunctionView *) view;
    if (tfv->function_name.empty() || tfv->name.empty()) {
        return DuckDBError;
    }
    conn.TableFunction(tfv->function_name, tfv->parameters)->CreateView(tfv->name, true, false);
    return DuckDBSuccess;
}

void duckdb_destroy_table_function_view(duckdb_table_function_view *view) {
    if (view && *view) {
        auto tfv = (bridge::CTableFunctionView *) *view;
        delete tfv;
        *view = nullptr;
    }
}

}
