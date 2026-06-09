// TODO: Implement BASCAL library and object linking.
//
// Dependency declarations such as `require com.bascal.sort.bubbleSort%` select
// linker inputs only. They do not create modules, namespaces, exports, or
// runtime-qualified symbols. The generated Microsoft BASIC symbol remains the
// global function name, for example `bubbleSort%`.
//
// Future work:
// - search -I and -L paths captured by the CLI
// - load .bcl/.ram sources and library archives
// - detect duplicate global BASIC symbols across transitive dependencies
// - reject transitive recursion before function lowering
// - lower function parameters to function-prefixed global variables
