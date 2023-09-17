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

    struct CScalarBindData : public FunctionData {
        ~CScalarBindData() override {
            if (bind_data && delete_callback) {
                delete_callback(bind_data);
            }
            bind_data = nullptr;
            delete_callback = nullptr;
        }

        unique_ptr<FunctionData> Copy() const override {
            throw InternalException("Copy not supported for CScalarBindData");
        }

        bool Equals(const FunctionData &other_p) const override {
            return false;
        }

        void *bind_data = nullptr;
        duckdb_delete_callback_t delete_callback = nullptr;

//        static unique_ptr<FunctionData> RegexpBind(
//                ClientContext &context,
//                ScalarFunction &bound_function,
//                vector<unique_ptr<Expression>> &arguments) {
//            // pattern is the second argument. If its constant, we can already prepare the pattern and store it for later.
//            D_ASSERT(arguments.size() == 2);
//            jfr_ext_re2::RE2::Options options;
//            options.set_log_errors(false);
//
//            string constant_string;
//            bool constant_pattern;
//            constant_pattern = TryParseConstantPattern(context, *arguments[1], constant_string);
//            return make_uniq<RegexpBindData>(options, std::move(constant_string), constant_pattern);
//        }

//        static bool TryParseConstantPattern(ClientContext &context, Expression &expr, string &constant_string) {
//            if (!expr.IsFoldable()) {
//                return false;
//            }
//            Value pattern_str = ExpressionExecutor::EvaluateScalar(context, expr);
//            if (!pattern_str.IsNull() && pattern_str.type().id() == LogicalTypeId::VARCHAR) {
//                constant_string = StringValue::Get(pattern_str);
//                return true;
//            }
//            return false;
//        }
    };

    struct CScalarInitData : public FunctionLocalState {
        ~CScalarInitData() override {
            if (init_data && delete_callback) {
                delete_callback(init_data);
            }
            init_data = nullptr;
            delete_callback = nullptr;
        }

//        explicit RegexpLocalState(CScalarBindData &info)
//                : constant_pattern(jfr_ext_re2::StringPiece(info.constant_string.c_str(), info.constant_string.size()),
//                                   info.options) {
//            D_ASSERT(info.constant_pattern);
//        }

//        jfr_ext_re2::RE2 constant_pattern;

        void *init_data = nullptr;
        duckdb_delete_callback_t delete_callback = nullptr;
    };

    struct CScalarInternalFunctionInfo {
        CScalarInternalFunctionInfo(CScalarBindData &bind_data, CScalarInitData &init_data)
                : bind_data(bind_data), init_data(init_data), success(true) {}

        CScalarBindData &bind_data;
        CScalarInitData &init_data;
        bool success;
        string error;
    };

    unique_ptr<FunctionData> CScalarFunctionBind(
            ClientContext &context,
            ScalarFunction &bound_function,
            vector<unique_ptr<Expression>> &arguments) {
//        if (dynamic_cast<CScalarFunctionDef *>(&bound_function) == nullptr) {
//            throw Exception("CScalarFunctionBind: bound_function is not a CScalarFunctionDef");
//        }
//        auto sf = reinterpret_cast<CScalarFunctionDef *>(&bound_function);
//
//        CScalarInternalBindInfo bind_info(*bind_data, arguments);
//        sf->bind(&bind_info);
//
//        if (!bind_info.success) {
//            throw Exception(bind_info.error);
//        }
        auto bind_data = make_uniq<CScalarBindData>();
        return std::move(bind_data);

//        // pattern is the second argument. If its constant, we can already prepare the pattern and store it for later.
//        D_ASSERT(arguments.size() == 2 || arguments.size() == 3);
//        RE2::Options options;
//        options.set_log_errors(false);
//        if (arguments.size() == 3) {
//            ParseRegexOptions(context, *arguments[2], options);
//        }
//
//        string constant_string;
//        bool constant_pattern;
//        constant_pattern = TryParseConstantPattern(context, *arguments[1], constant_string);
//        return make_uniq<RegexpMatchesBindData>(options, std::move(constant_string), constant_pattern);
//        return nullptr;
    }

    unique_ptr<FunctionLocalState> CScalarFunctionInit(
            ExpressionState &state,
            const BoundFunctionExpression &expr,
            FunctionData *bind_data) {
        auto init_data = make_uniq<CScalarInitData>();
        return std::move(init_data);
    }
}

extern "C" {

duckdb_scalar_function duckdb_create_scalar_function() {
    auto function = new duckdb::ScalarFunction(
            "", {},
            duckdb::LogicalType::INVALID,
            nullptr, bridge::CScalarFunctionBind, nullptr, nullptr, bridge::CScalarFunctionInit,
            duckdb::LogicalType::INVALID,
            duckdb::FunctionSideEffects::NO_SIDE_EFFECTS,
            duckdb::FunctionNullHandling::SPECIAL_HANDLING);
    return function;
}

void duckdb_destroy_scalar_function(duckdb_scalar_function *function) {
    if (function && *function) {
        auto sf = (duckdb::ScalarFunction * ) * function;
        delete sf;
        *function = nullptr;
    }
}

void duckdb_scalar_function_set_name(duckdb_scalar_function function, const char *name) {
    if (!function || !name) {
        return;
    }
    auto sf = (duckdb::ScalarFunction *) function;
    sf->name = name;
}

void duckdb_scalar_function_set_return_type(duckdb_scalar_function function, duckdb_logical_type type) {
    if (!function || !type) {
        return;
    }
    auto sf = (duckdb::ScalarFunction *) function;
    auto logical_type = (duckdb::LogicalType *) type;
    sf->return_type = *logical_type;
}

void duckdb_scalar_function_add_parameter(duckdb_scalar_function function, duckdb_logical_type type) {
    if (!function || !type) {
        return;
    }
    auto sf = (duckdb::ScalarFunction *) function;
    auto logical_type = (duckdb::LogicalType *) type;
    sf->arguments.push_back(*logical_type);
}

//void duckdb_scalar_function_set_bind(duckdb_scalar_function scalar_function, duckdb_scalar_function_bind_t bind) {
//    if (!scalar_function || !bind) {
//        return;
//    }
//    auto sf = (duckdb::ScalarFunction *) scalar_function;
//    sf->bind = [bind](
//            duckdb::ClientContext &context,
//            duckdb::ScalarFunction &bound_function,
//            duckdb::vector<duckdb::unique_ptr<duckdb::Expression>> &arguments) -> duckdb::unique_ptr<duckdb::FunctionData> {
//        auto bind_data = duckdb::make_uniq<bridge::CScalarBindData>();
//        bridge::CScalarInternalBindInfo bind_info(*bind_data, arguments);
//        bind(&bind_info);
//
//        if (!bind_info.success) {
//            throw duckdb::Exception(bind_info.error);
//        }
//
//        return std::move(bind_data);
//    };
//}
//
//void duckdb_scalar_function_set_init(duckdb_scalar_function scalar_function, duckdb_scalar_function_init_t init) {
//    if (!scalar_function || !init) {
//        return;
//    }
//    auto sf = (bridge::ScalarFunction *) scalar_function;
//    sf->init_local_state = [init](
//            duckdb::ExpressionState &state,
//            const duckdb::BoundFunctionExpression &expr,
//            duckdb::FunctionData *bind_data) -> duckdb::unique_ptr<duckdb::FunctionLocalState> {
//        auto init_data = duckdb::make_uniq<bridge::CScalarInitData>();
//        bridge::CScalarInternalInitInfo init_info(*init_data);
//        init(&init_info);
//
//        if (!init_info.success) {
//            throw duckdb::Exception(init_info.error);
//        }
//        return std::move(init_data);
//    };
//}

void duckdb_scalar_function_set_function(duckdb_scalar_function scalar_function, duckdb_scalar_function_t function) {
    if (!scalar_function || !function) {
        return;
    }
    auto sf = (bridge::ScalarFunction *) scalar_function;
    sf->function = [=](
            duckdb::DataChunk &args,
            duckdb::ExpressionState &state,
            duckdb::Vector &result) {
        auto &func_expr = state.expr.Cast<duckdb::BoundFunctionExpression>();
        auto &bind_data = (bridge::CScalarBindData &) *func_expr.bind_info;
        auto &lstate = (duckdb::ExecuteFunctionState &) state;

        bridge::CScalarInternalFunctionInfo function_info(bind_data, (bridge::CScalarInitData &) *lstate.local_state);

        function(
                &function_info,
                reinterpret_cast<duckdb_data_chunk>(&args),
                reinterpret_cast<duckdb_vector>(&result));

        if (!function_info.success) {
            throw duckdb::Exception(function_info.error);
        }
    };
}

void duckdb_scalar_function_set_bind_data(duckdb_scalar_function_info info, void *extra_data, duckdb_delete_callback_t destroy) {
    if (!info) {
        return;
    }
    auto bind_info = (bridge::CScalarInternalFunctionInfo *)info;
    bind_info->bind_data.bind_data = extra_data;
    bind_info->bind_data.delete_callback = destroy;
}

void duckdb_scalar_function_set_init_data(duckdb_scalar_function_info info, void *extra_data, duckdb_delete_callback_t destroy) {
    if (!info) {
        return;
    }
    auto init_info = (bridge::CScalarInternalFunctionInfo *)info;
    init_info->init_data.init_data = extra_data;
    init_info->init_data.delete_callback = destroy;
}

void *duckdb_scalar_function_get_bind_data(duckdb_scalar_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (bridge::CScalarInternalFunctionInfo *)info;
    return function_info->bind_data.bind_data;
}

void *duckdb_scalar_function_get_init_data(duckdb_scalar_function_info info) {
    if (!info) {
        return nullptr;
    }
    auto function_info = (bridge::CScalarInternalFunctionInfo *)info;
    return function_info->init_data.init_data;
}

void duckdb_scalar_function_set_error(duckdb_scalar_function_info info, const char *error) {
    if (!info || !error) {
        return;
    }
    auto function_info = (bridge::CScalarInternalFunctionInfo *)info;
    function_info->error = error;
    function_info->success = false;
}

duckdb_state duckdb_register_scalar_function(duckdb_connection connection, duckdb_scalar_function function) {
    if (!connection || !function) {
        return DuckDBError;
    }
    auto con = (duckdb::Connection *) connection;
    auto sf = (bridge::ScalarFunction *) function;
    if (sf->name.empty() || sf->return_type == duckdb::LogicalType::INVALID || !sf->function) {
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

}
