Diagnostic preparation: the first temporary-manifest patch had a missing patch
marker and was rejected before writing. Its corrected form resolved offline.
The first identity extractor assumed Cargo's optional `.cargo-checksum.json`
existed; it did not. The final extractor verifies every extracted file directly
against the exact archive, whose checksum matches the lock and fresh index.
A first one-line receipt-verification command had an unmatched brace; the
structured `verify.nu` replaces it. These preparation errors executed no failed
Cargo gate and changed no production input. The raw compiler/test/package
receipts record every actual gate outcome without substituting these attempts.
