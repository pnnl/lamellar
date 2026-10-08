# Run one launcher command and check it the way the old smoke scripts did: the exit status is 0, the success marker appears
# exactly EXPECT times (once per PE that prints it), and no failure text appears.
#   cmake -DRUN=<oshrun> -DLAUNCH="-np 4" -DPROG=<program> -DEXPECT=4 -DPATTERN=<regex> [-DTIMEOUT=<s>] -P expect.cmake
cmake_minimum_required(VERSION 3.20)
separate_arguments(LAUNCH)
if(NOT DEFINED TIMEOUT OR TIMEOUT STREQUAL "")
  set(TIMEOUT 240)
endif()

execute_process(COMMAND ${RUN} ${LAUNCH} ${PROG}
  RESULT_VARIABLE rc OUTPUT_VARIABLE out ERROR_VARIABLE err TIMEOUT ${TIMEOUT})
set(all "${out}\n${err}")

string(REGEX MATCHALL "${PATTERN}" found "${all}")
list(LENGTH found nfound)
string(REGEX MATCHALL "FAIL|panicked|InitError|not registered|neither" bad "${all}")
list(LENGTH bad nbad)

if(NOT rc STREQUAL "0" OR NOT nfound EQUAL EXPECT OR nbad GREATER 0)
  message("${all}")
  message(FATAL_ERROR "${PROG}: exit status '${rc}', marker '${PATTERN}' seen ${nfound}/${EXPECT} times, ${nbad} failure message(s)")
endif()
message(STATUS "${nfound}/${EXPECT} PEs passed")
