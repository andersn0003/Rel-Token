    soroban contract invoke \
--wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
--id CB4AQGALS6MB4BBH75EC4F2KE2BERJPUE77XCF5ONISPYD2G5W32LBFT \
    --source thor \
    --network futurenet \
    -- \
    get_documents


    soroban contract invoke \
    --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
    --id CCX7XGMKI6MERSDZXMUTHHWTPNKRBMFKSKR2ZA4DQ6WCAUSNZVZEVJZG \
    --source thor \
    --network futurenet \
    -- \
    get_document \
    --doc_id 18