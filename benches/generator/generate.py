import os
import random
import string


def generator(
    total_files: int = 1000,
    max_lines: int = 2000,
    out_dir_name: str = "test_data",
) -> None:
    """Random Data Generator for benchmarking purposes"""

    os.makedirs(out_dir_name, exist_ok=True)

    total_bytes = 0
    total_lines = 0

    print(f"Generating {total_files} files in '{out_dir_name}/'...")

    for i in range(total_files):
        file_name = os.path.join(out_dir_name, f"test_module_{i+1}.hm")

        with open(file_name, "w", encoding="utf-8") as f:
            lines = []
            num_lines = random.randint(100, max_lines)
            total_lines += num_lines

            for _ in range(num_lines):
                line_len = random.randint(10, 100)
                line = "".join(
                    random.choices(string.ascii_letters + " {}[];:=+-", k=line_len)
                )
                lines.append(line)

            content = "\n".join(lines)
            f.write(content)
            total_bytes += len(content)

    print(
        f"Done! Generated {total_lines} lines across {total_bytes / 1024 / 1024:.2f} MB of data."
    )


if __name__ == "__main__":
    generator()
