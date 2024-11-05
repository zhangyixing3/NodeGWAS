import sys

def process_sigsite_file(sigsite_file):
    """
    Read the Elongation2.result.ps.1_sigSite.out file and create a hash table
    """
    hash_table = {}
    with open(sigsite_file, 'r') as f:
        for line in f:
            # Split the line by tab or whitespace
            columns = line.split()
            if len(columns) < 4:
                continue
            hash_table[columns[0]] = columns[3]
    return hash_table


def process_bubble_file(node_file, hash_table):
    """
    Read the node file and print relevant data if the 3rd column is in the hash table.
    """
    with open(node_file, 'r') as f:
        for line in f:
            columns = line.split()
            if len(columns) != 7:
                break
            node = columns[2]  # 3rd column in 0-based index
            # Check if third_column exists in hash_table
            if node in hash_table:
                print(f"{columns[0]}\t{columns[5]}\t{columns[6]}\t{node}\t{hash_table[node]}")


def process_ref_result_file(ref_result_file, hash_table):
    """
    Read the ref_result file and print relevant data if the 2nd column is in the hash table.
    """
    with open(ref_result_file, 'r') as f:
        for line in f:
            columns = line.split()
            if len(columns) != 3:
                break
            node = columns[1]  # 2nd column in 0-based index
            # Check if second_column exists in hash_table
            if node in hash_table:
                print(f"{columns[0]}\t{columns[2]}\t{columns[2]}\t{node}\t{hash_table[node]}")


def main():
    # Ensure the script is called with the correct number of arguments
    if len(sys.argv) != 4:
        print("Usage: python emmax_node_position.py <significant_sites> <bubble(7 columns)> <ref_result(3 columns)>")
        sys.exit(1)

    sigsite_file = sys.argv[1]    # Elongation2.result.ps.1_sigSite.out
    node_file = sys.argv[2]       # node
    ref_result_file = sys.argv[3] # ref_result

    # Step 1: Process the sigsite file to create the hash table
    hash_table = process_sigsite_file(sigsite_file)

    # Step 2: Process the node file based on the hash table
    process_bubble_file(node_file, hash_table)

    # Step 3: Process the ref_result file based on the hash table
    process_ref_result_file(ref_result_file, hash_table)


if __name__ == '__main__':
    main()
