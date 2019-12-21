file(GLOB HEADERS "include/eosio/bridge_plugin/*.hpp")

# build rust based bifrost rpc
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_CMD cargo build --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/bifrost_rpc/ffi/Cargo.toml)
    set(TARGET_DIR "debug")
else ()
    set(CARGO_CMD cargo build --release --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/bifrost_rpc/ffi/Cargo.toml)
    set(TARGET_DIR "release")
endif ()

set(LIB_RPC "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/librpc_client.dylib")

add_custom_command(
    OUTPUT bifrost_rpc
    PRE_BUILD
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} ${CARGO_CMD}
    COMMAND cp ${LIB_RPC} ${CMAKE_CURRENT_BINARY_DIR}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Compiled rust based bifrost rpc module successfully "
)

add_library(
    bridge_plugin
    bridge_plugin.cpp
    ${HEADERS}
)

target_link_libraries(
        bridge_plugin
        chain_plugin
        appbase
        ${LIB_RPC}
)
target_include_directories(
        bridge_plugin PUBLIC
        "${CMAKE_CURRENT_SOURCE_DIR}/include"
        "${CMAKE_CURRENT_SOURCE_DIR}/bifrost_rpc"
        "${CMAKE_CURRENT_SOURCE_DIR}/../chain_interface/include"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../libraries/appbase/include"
)
