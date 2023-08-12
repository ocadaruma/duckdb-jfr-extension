.PHONY: all clean format debug release duckdb_debug duckdb_release pull update

all: release

MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
PROJ_DIR := $(dir $(MKFILE_PATH))

OSX_ARCH_FLAG=
ifneq (${OSX_BUILD_ARCH}, "")
	OSX_ARCH_FLAG=-DOSX_BUILD_ARCH=${OSX_BUILD_ARCH}
endif

ifeq ($(GEN),ninja)
	GENERATOR=-G "Ninja"
	FORCE_COLOR=-DFORCE_COLORED_OUTPUT=1
endif

BUILD_FLAGS=-DEXTENSION_STATIC_BUILD=1 -DBUILD_EXTENSIONS="tpch" ${OSX_ARCH_FLAG}

CLIENT_FLAGS :=

# These flags will make DuckDB build the extension
EXTENSION_FLAGS= \
-DDUCKDB_EXTENSION_NAMES="jfr" \
-DDUCKDB_EXTENSION_JFR_PATH="$(PROJ_DIR)" \
-DDUCKDB_EXTENSION_JFR_SHOULD_LINK=0
#-DDUCKDB_EXTENSION_JFR_LOAD_TESTS=1 \
#-DDUCKDB_EXTENSION_JFR_TEST_PATH=$(PROJ_DIR)test \
#-DDUCKDB_EXTENSION_JFR_INCLUDE_PATH="$(PROJ_DIR)src/include" \

pull:
	git submodule init
	git submodule update --recursive --remote

clean:
	rm -rf build
	cd duckdb && make clean

# Main build
debug:
	mkdir -p  build/debug && \
	cmake $(GENERATOR) $(FORCE_COLOR) $(EXTENSION_FLAGS) ${CLIENT_FLAGS} -DEXTENSION_STATIC_BUILD=1 -DCMAKE_BUILD_TYPE=Debug ${BUILD_FLAGS} -S ./duckdb/ -B build/debug && \
	cmake --build build/debug --config Debug

release:
	mkdir -p build/release && \
	cmake $(GENERATOR) $(FORCE_COLOR) $(EXTENSION_FLAGS) ${CLIENT_FLAGS} -DEXTENSION_STATIC_BUILD=1 -DCMAKE_BUILD_TYPE=Release ${BUILD_FLAGS} -S ./duckdb/ -B build/release && \
	cmake --build build/release --config Release

#test: test_release
#
#test_release: release
#	./build/release/test/unittest "$(PROJ_DIR)test/*"
#
#test_debug: debug
#	./build/release/test/unittest "$(PROJ_DIR)test/*"

#format:
#	find src/ -iname *.hpp -o -iname *.cpp | xargs clang-format --sort-includes=0 -style=file -i
#	cmake-format -i CMakeLists.txt

update:
	git submodule update --remote --merge
