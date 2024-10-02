# UEFI Wrapper for binary loaders

Testing tool to add support for UEFI to Microkit. Used in conjunction with: https://github.com/Kswin01/efi_section_inject to
inject sections into an EFI image, which is then loaded into a specific address that the Microkit loader image is expecting to be
placed at (the microkit loader is currently position dependent, and must be placed at the address as specified for the board in
the microkit `build_sdk.py` script.
