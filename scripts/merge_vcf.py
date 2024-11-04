import argparse
import gzip

def merge_vcf_files(vcf_files, output_file):
    """Merge multiple VCF files into one, retaining the header from the first file."""
    first_file = True
    with open(output_file, 'w') as out_f:
        for vcf_file in vcf_files:
            with gzip.open(vcf_file, 'rt') as in_f:
                for line in in_f:
                    if line.startswith('#'):
                        if first_file:
                            out_f.write(line)
                    else:
                        out_f.write(line)
                first_file = False

    print(f"Merged VCF files into {output_file}")

if __name__ == "__main__":
    # Parse command-line arguments
    parser = argparse.ArgumentParser(description='Merge multiple VCF files, retaining the header from the first file.')
    parser.add_argument('vcf_files', metavar='VCF_FILE', type=str, nargs='+',
                        help='List of VCF files to merge')
    parser.add_argument('-o', '--output', metavar='OUTPUT_FILE', type=str, required=True,
                        help='Output merged VCF file')

    args = parser.parse_args()

    # Perform merging
    merge_vcf_files(args.vcf_files, args.output)

