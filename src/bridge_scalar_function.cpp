#include "bridge.hpp"
#include "duckdb.hpp"
#include "duckdb/parser/parsed_data/create_scalar_function_info.hpp"

namespace bridge {
    using namespace duckdb;

    struct CScalarInternalFunctionInfo {
        CScalarInternalFunctionInfo() : success(true) {}
        bool success;
        string error;
    };
}

extern "C" {

duckdb_scalar_function duckdb_create_scalar_function() {
    auto function = new duckdb::ScalarFunction(
            "", {},
            duckdb::LogicalType::INVALID,
            nullptr, nullptr, nullptr, nullptr, nullptr,
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

void duckdb_scalar_function_set_function(duckdb_scalar_function scalar_function, duckdb_scalar_function_t function) {
    if (!scalar_function || !function) {
        return;
    }
    auto sf = (bridge::ScalarFunction *) scalar_function;
    sf->function = [=](
            duckdb::DataChunk &args,
            duckdb::ExpressionState &state,
            duckdb::Vector &result) {
        bridge::CScalarInternalFunctionInfo function_info;

        result.SetVectorType(duckdb::VectorType::FLAT_VECTOR);
        function(
                &function_info,
                reinterpret_cast<duckdb_data_chunk>(&args),
                reinterpret_cast<duckdb_vector>(&result));
        if (!function_info.success) {
            throw duckdb::Exception(function_info.error);
        }
        if (args.AllConstant()) {
            result.SetVectorType(duckdb::VectorType::CONSTANT_VECTOR);
        }
    };
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
