#ifndef DUCKDB_BUILD_LOADABLE_EXTENSION
#define DUCKDB_BUILD_LOADABLE_EXTENSION
#endif
#include "duckdb.h"


extern "C" {
typedef void *duckdb_client_context;
typedef struct _duckdb_unified_vector_format {
    void *__duvf;
} * duckdb_unified_vector_format;

//===--------------------------------------------------------------------===//
// View API
//===--------------------------------------------------------------------===//
typedef void *duckdb_table_function_view;

/*!
Creates a new empty table function view.

The return value should be destroyed with `duckdb_destroy_table_function_view`.

* returns: The table function view object.
*/
duckdb_table_function_view duckdb_create_table_function_view();

/*!
Sets the name of table function to be called by this view.

* view: The table function view
* name: The name of the table function
*/
void duckdb_table_function_view_set_function_name(duckdb_table_function_view view, const char *function_name);

/*!
Sets the name of the view.

* view: The table function view
* name: The name of the view
*/
void duckdb_table_function_view_set_name(duckdb_table_function_view view, const char *name);

/*!
Adds a parameter to the table function to be called by this view.

* view: The table function view
* value: The value of the parameter to add.
*/
void duckdb_table_function_view_add_parameter(duckdb_table_function_view view, duckdb_value value);

/*!
Register the table function view.

* context: The client context to register it in.
* view: The table function view object
* returns: Whether or not the registration was successful.
*/
duckdb_state duckdb_register_table_function_view(duckdb_client_context context, duckdb_table_function_view view);

/*!
Destroys the given table function view object.

* view: The table function view to destroy
*/
void duckdb_destroy_table_function_view(duckdb_table_function_view *view);

//===--------------------------------------------------------------------===//
// Logical Type Interface
//===--------------------------------------------------------------------===//
duckdb_logical_type duckdb_create_struct_type2(
        idx_t n_pairs,
        const char** names,
        const duckdb_logical_type* types);

//===--------------------------------------------------------------------===//
// Table Functions
// These are modified version of original duckdb C APIs to support
// init/bind/function variants which accepts ClientContext
//===--------------------------------------------------------------------===//
typedef void (*duckdb_table_function2_bind_t)(duckdb_client_context ctx, duckdb_bind_info info);
typedef void (*duckdb_table_function2_t)(duckdb_client_context ctx, duckdb_function_info info, duckdb_data_chunk output);

/*!
Creates a new empty table function.

The return value should be destroyed with `duckdb_destroy_table_function`.

* returns: The table function object.
*/
duckdb_table_function duckdb_create_table_function2();

/*!
Sets the main function of the table function

* table_function: The table function
* function: The function
*/
void duckdb_table_function2_set_function(duckdb_table_function table_function, duckdb_table_function2_t function);

/*!
Sets the bind function of the table function

* table_function: The table function
* bind: The bind function
*/
void duckdb_table_function2_set_bind(duckdb_table_function table_function, duckdb_table_function2_bind_t bind);

/*!
Sets the init function of the table function

* table_function: The table function
* init: The init function
*/
void duckdb_table_function2_set_init(duckdb_table_function table_function, duckdb_table_function_init_t init);

/*!
Register the table function object within the given connection.

The function requires at least a name, a bind function, an init function and a main function.

If the function is incomplete or a function with this name already exists DuckDBError is returned.

* con: The connection to register it in.
* function: The function pointer
* returns: Whether or not the registration was successful.
*/
duckdb_state duckdb_register_table_function2(duckdb_connection connection, duckdb_table_function function);

/*!
Gets the bind data set by `duckdb_bind_set_bind_data` during the bind.

Note that the bind data should be considered as read-only.
For tracking state, use the init data instead.

* info: The info object
* returns: The bind data object
*/
void *duckdb_function2_get_bind_data(duckdb_function_info info);

/*!
Gets the init data set by `duckdb_init_set_init_data` during the init.

* info: The info object
* returns: The init data object
*/
void *duckdb_function2_get_init_data(duckdb_function_info info);

//===--------------------------------------------------------------------===//
// File systems
//===--------------------------------------------------------------------===//
typedef void *duckdb_file_handle;

/*!
Open a file handle to a file with the given path and mode.

The return value should be closed with `duckdb_file_close`.

 * context: The client context
 * path: The path to the file
 * flags: The flags to open the file with
 * returns: The file handle
 */
duckdb_file_handle duckdb_open_file(duckdb_client_context context, const char *path, uint8_t flags);

/*!
Get the size of the file in bytes.

 * handle: The file handle
 * returns: The size of the file in bytes
 */
int64_t duckdb_file_get_size(duckdb_file_handle handle);

/*!
Read data from a file handle into a buffer.

 * handle: The file handle
 * buffer: The buffer to read into
 * nr_bytes: The number of bytes to read
 * returns: The number of bytes read
 */
int64_t duckdb_file_read(duckdb_file_handle handle, void *buffer, int64_t nr_bytes);

/*!
Seek to a position in the file.

 * handle: The file handle
 * pos: The position to seek to
 */
void duckdb_file_seek(duckdb_file_handle handle, idx_t pos);

/*!
Close the file handle.

 * handle: The file handle
 */
void duckdb_file_close(duckdb_file_handle handle);

//===--------------------------------------------------------------------===//
// Scalar Functions
//===--------------------------------------------------------------------===//
typedef void *duckdb_scalar_function;
typedef void *duckdb_scalar_function_info;
typedef void (*duckdb_scalar_function_t)(
        duckdb_scalar_function_info info,
        duckdb_data_chunk args,
        duckdb_vector result);

/*!
Creates a new empty scalar function.

The return value should be destroyed with `duckdb_destroy_scalar_function`.

* returns: The scalar function object.
*/
duckdb_scalar_function duckdb_create_scalar_function();

/*!
Destroys the given scalar function object.

* function: The scalar function to destroy
*/
void duckdb_destroy_scalar_function(duckdb_scalar_function *function);

/*!
Sets the name of the given scalar function.

* function: The scalar function
* name: The name of the scalar function
*/
void duckdb_scalar_function_set_name(duckdb_scalar_function function, const char *name);

/*!
Sets the return type of the given scalar function.

* function: The scalar function
* type: The return type of the scalar function
*/
void duckdb_scalar_function_set_return_type(duckdb_scalar_function function, duckdb_logical_type type);

/*!
Adds a parameter to the scalar function.

* function: The scalar function
* type: The type of the parameter to add.
*/
void duckdb_scalar_function_add_parameter(duckdb_scalar_function function, duckdb_logical_type type);

/*!
Sets the main function of the scalar function

* scalar_function: The scalar function
* function: The function
*/
void duckdb_scalar_function_set_function(duckdb_scalar_function scalar_function, duckdb_scalar_function_t function);

/*!
Register the scalar function object within the given connection.

The function requires at least a name, a return type and a main function.

If the function is invalid DuckDBError is returned.

* connection: The connection to register it in.
* function: The function pointer
* returns: Whether or not the registration was successful.
*/
duckdb_state duckdb_register_scalar_function(duckdb_connection connection, duckdb_scalar_function function);

/*!
Report that an error has occurred while executing the function.

* info: The info object
* error: The error message
*/
void duckdb_scalar_function_set_error(duckdb_scalar_function_info info, const char *error);

//===--------------------------------------------------------------------===//
// C APIs for strings
//===--------------------------------------------------------------------===//
typedef struct {
    const char *data;
    idx_t size;
} string_piece;

/*!
Get a string piece from a vector.

The returned char pointer MUST not be freed.

 * vector: The vector to get the string piece from
 * index: The index in the vector
 * returns: The string piece
 */
string_piece duckdb_get_string(duckdb_unified_vector_format vector, idx_t index);

//===--------------------------------------------------------------------===//
// Vector
//===--------------------------------------------------------------------===//
/*!
Returns whether or not a row is valid (i.e. not NULL) in a vector.

 * vector: The vector
 * row: The row index
 * returns: true if the row is valid, false otherwise
 */
bool duckdb_unified_vector_validity_row_is_valid(duckdb_unified_vector_format vector, idx_t row);

/*!
Converts a duckdb vector to unified vector format.
Returned vector must be freed with `duckdb_destroy_unified_vector_format`.

 * vector: The vector
 * size: The size of the vector
 * returns: The unified vector format
 */
duckdb_unified_vector_format duckdb_to_unified_format(duckdb_vector vector, idx_t size);

/*!
Destroys the given unified vector format object.

 * vector: The vector
 */
void duckdb_destroy_unified_vector_format(duckdb_unified_vector_format *vector);

/*!
Returns whether or not a vector is constant.

 * vector: The vector
 * returns: true if the vector is constant, false otherwise
 */
bool duckdb_vector_is_constant(duckdb_vector vector);

}
