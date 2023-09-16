#include "duckdb.hpp"
#include "duckdb/main/capi/capi_internal.hpp"
#include "duckdb/function/scalar_function.hpp"
//#include "duckdb/function/scalar/regexp.hpp"
#include "duckdb/common/assert.hpp"
#include "duckdb/function/table_function.hpp"
#include "duckdb/parser/parsed_data/create_table_function_info.hpp"
#include "duckdb/parser/parsed_data/create_scalar_function_info.hpp"
#include "duckdb/planner/expression/bound_function_expression.hpp"
#include "duckdb/execution/expression_executor.hpp"
#include "duckdb/common/helper.hpp"
#include "re2/re2.h"
#include "bridge.hpp"

#include <iostream>
#include <memory>

using duckdb::Connection;
using duckdb::DuckDB;
using duckdb::Value;

/// Taken from table_function-c.cpp

namespace bridge {
    using namespace duckdb;

    struct RegexpBindData : public FunctionData {
        RegexpBindData(jfr_ext_re2::RE2::Options options, string constant_string, bool constant_pattern)
                : options(options), constant_string(std::move(constant_string)), constant_pattern(constant_pattern) {
            if (constant_pattern) {
                auto pattern = make_uniq<jfr_ext_re2::RE2>(constant_string, options);
                if (!pattern->ok()) {
                    throw Exception(pattern->error());
                }
            }
        }

        jfr_ext_re2::RE2::Options options;
        string constant_string;
        bool constant_pattern;

        unique_ptr<FunctionData> Copy() const override {
            return make_uniq<RegexpBindData>(options, constant_string, constant_pattern);
        }

        bool Equals(const FunctionData &other_p) const {
            auto &other = other_p.Cast<RegexpBindData>();
            return constant_pattern == other.constant_pattern && constant_string == other.constant_string &&
                   RegexOptionsEquals(options, other.options);
        }

        static bool RegexOptionsEquals(const jfr_ext_re2::RE2::Options &opt_a, const jfr_ext_re2::RE2::Options &opt_b) {
            return opt_a.case_sensitive() == opt_b.case_sensitive();
        }

        static unique_ptr<FunctionData> RegexpBind(
                ClientContext &context,
                ScalarFunction &bound_function,
                vector<unique_ptr<Expression>> &arguments) {
            // pattern is the second argument. If its constant, we can already prepare the pattern and store it for later.
            D_ASSERT(arguments.size() == 2);
            jfr_ext_re2::RE2::Options options;
            options.set_log_errors(false);

            string constant_string;
            bool constant_pattern;
            constant_pattern = TryParseConstantPattern(context, *arguments[1], constant_string);
            return make_uniq<RegexpBindData>(options, std::move(constant_string), constant_pattern);
        }

        static bool TryParseConstantPattern(ClientContext &context, Expression &expr, string &constant_string) {
            if (!expr.IsFoldable()) {
                return false;
            }
            Value pattern_str = ExpressionExecutor::EvaluateScalar(context, expr);
            if (!pattern_str.IsNull() && pattern_str.type().id() == LogicalTypeId::VARCHAR) {
                constant_string = StringValue::Get(pattern_str);
                return true;
            }
            return false;
        }
    };

    inline jfr_ext_re2::StringPiece CreateStringPiece(const string_t &input) {
        return jfr_ext_re2::StringPiece(input.GetData(), input.GetSize());
    }

    struct RegexpLocalState : public FunctionLocalState {
        explicit RegexpLocalState(RegexpBindData &info)
                : constant_pattern(jfr_ext_re2::StringPiece(info.constant_string.c_str(), info.constant_string.size()),
                                   info.options) {
            D_ASSERT(info.constant_pattern);
        }

        jfr_ext_re2::RE2 constant_pattern;
    };

    unique_ptr<FunctionLocalState> RegexpInitState(ExpressionState &state, const BoundFunctionExpression &expr,
                                                   FunctionData *bind_data) {
        auto &info = bind_data->Cast<RegexpBindData>();
        if (info.constant_pattern) {
            return make_uniq<RegexpLocalState>(info);
        }
        return nullptr;
    }

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

    static void StacktraceMatchesFunction(DataChunk &args, ExpressionState &state, Vector &result) {
        auto &strings = args.data[0];
        auto &patterns = args.data[1];

        auto &func_expr = state.expr.Cast<BoundFunctionExpression>();
        auto &info = func_expr.bind_info->Cast<RegexpBindData>();

        if (info.constant_pattern) {
            auto fstate = reinterpret_cast<ExecuteFunctionState *>(&state);
            auto &lstate = reinterpret_cast<RegexpLocalState &>(*fstate->local_state);
            UnaryExecutor::Execute<string_t, bool>(strings, result, args.size(), [&](string_t input) {
                return jfr_ext_re2::RE2::PartialMatch(CreateStringPiece(input),
                                                      lstate.constant_pattern);
            });
        } else {
            BinaryExecutor::Execute<string_t, string_t, bool>(
                    strings, patterns, result, args.size(),
                    [&](string_t input, string_t pattern) {
                        jfr_ext_re2::RE2 re(CreateStringPiece(pattern), info.options);
                        if (!re.ok()) {
                            throw Exception(re.error());
                        }
                        return jfr_ext_re2::RE2::PartialMatch(CreateStringPiece(input), re);
                    });
        }
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

void jfr_scan_create_view(duckdb_client_context context, const char *filename, const char *tablename) {
    auto ctx = (duckdb::ClientContext *) context;
    auto conn = duckdb::Connection(ctx->db->GetDatabase(*ctx));
    conn.TableFunction("jfr_scan", {Value(filename), Value(tablename)})
            ->CreateView(tablename, true, false);
}

void jfr_register_stacktrace_matches_function(duckdb_connection connection) {
    duckdb::ScalarFunction sf("stacktrace_matches",
                              {duckdb::LogicalType::VARCHAR, duckdb::LogicalType::VARCHAR},
                              duckdb::LogicalType::BOOLEAN,
                              bridge::StacktraceMatchesFunction,
                              bridge::RegexpBindData::RegexpBind, nullptr, nullptr,
                              bridge::RegexpInitState,
                              duckdb::LogicalType::INVALID,
                              duckdb::FunctionSideEffects::NO_SIDE_EFFECTS,
                              duckdb::FunctionNullHandling::SPECIAL_HANDLING);
    duckdb::CreateScalarFunctionInfo sf_info(sf);
    auto con = (duckdb::Connection *) connection;
    con->context->RunFunctionInTransaction([&]() {
        auto &catalog = duckdb::Catalog::GetSystemCatalog(*con->context);
        catalog.CreateFunction(*con->context, sf_info);
    });
}

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

duckdb_scalar_function duckdb_create_scalar_function(const char *name, duckdb_logical_type return_type) {
    auto logical_type = (duckdb::LogicalType *) return_type;
    auto function = new duckdb::ScalarFunction(name, {}, *logical_type, nullptr);
    return function;
}

void duckdb_scalar_function_add_parameter(duckdb_scalar_function function, duckdb_logical_type type) {
    if (!function || !type) {
        return;
    }
    auto sf = (duckdb::ScalarFunction *) function;
    auto logical_type = (duckdb::LogicalType *) type;
    sf->arguments.push_back(*logical_type);
}

void duckdb_scalar_function_set_function(duckdb_scalar_function scalar_function, duckdb_scalar_function_t function) {
    if (!scalar_function || !function) {
        return;
    }
    auto sf = (duckdb::ScalarFunction *) scalar_function;
    sf->function = [function](duckdb::DataChunk &args, duckdb::ExpressionState &state, duckdb::Vector &result) {
        function(reinterpret_cast<duckdb_data_chunk>(&args), &state, reinterpret_cast<duckdb_vector>(&result));
    };
}

duckdb_state duckdb_register_scalar_function(duckdb_connection connection, duckdb_scalar_function function) {
    if (!connection || !function) {
        return DuckDBError;
    }
    auto con = (duckdb::Connection *) connection;
    auto sf = (duckdb::ScalarFunction *) function;
    if (sf->name.empty() || !sf->function) {
        return DuckDBError;
    }
    con->context->RunFunctionInTransaction([&]() {
        auto &catalog = duckdb::Catalog::GetSystemCatalog(*con->context);
        duckdb::CreateScalarFunctionInfo sf_info(*sf);

        // create the function in the catalog
        catalog.CreateFunction(*con->context, sf_info);
    });
    return DuckDBSuccess;
}

void duckdb_destroy_scalar_function(duckdb_scalar_function *function) {
    if (function && *function) {
        auto sf = (duckdb::ScalarFunction * ) * function;
        delete sf;
        *function = nullptr;
    }
}

string_piece duckdb_get_string(duckdb_vector vector, idx_t index) {
    auto data = duckdb::ConstantVector::GetData<duckdb::string_t>((duckdb::Vector &) vector)[index];
    return string_piece{data.GetData(), data.GetSize()};
}

}
