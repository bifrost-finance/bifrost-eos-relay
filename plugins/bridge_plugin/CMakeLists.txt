file(GLOB HEADERS "include/eosio/bridge_plugin/*.hpp")
add_library( bridge_plugin
             bridge_plugin.cpp
             ${HEADERS} )

target_link_libraries( bridge_plugin chain_plugin appbase )
target_include_directories( bridge_plugin PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include" "${CMAKE_CURRENT_SOURCE_DIR}/../chain_interface/include" "${CMAKE_CURRENT_SOURCE_DIR}/../../libraries/appbase/include")
