package ham

import "core:fmt"
import "core:os"

errorf :: proc(format: string, args: ..any) {
	fmt.fprintf(os.stderr, "\x1b[31mError:\x1b[0m %s\n", fmt.tprintf(format, ..args))
}

main :: proc() {
	if len(os.args) < 2 {
		errorf("Please provide a file to execute.")
		os.exit(1)
	}

	filename := os.args[1]
	data, ok := os.read_entire_file(filename, context.allocator)
	if ok != nil {
		errorf("Can't read file: %s", filename)
		os.exit(1)
	}
}
