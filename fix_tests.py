import re

# Read the file
with open('src/tests/final_push_tests.rs', 'r') as f:
    content = f.read()

# Fix field names
content = content.replace('config.num_persons', 'config.entity_count')
content = content.replace('config.enable_trade_fees', 'config.transaction_fee')
content = content.replace('config.trade_fee_rate', 'config.transaction_fee')
content = content.replace('config.enable_income_tax', 'config.tax_rate')
content = content.replace('config.income_tax_rate', 'config.tax_rate')
content = content.replace('config.price_floor', 'config.min_skill_price')
content = content.replace('config.price_ceiling', 'config.max_skill_price')

# Fix TransactionType enum values
content = content.replace('TransactionType::Purchase', 'TransactionType::Buy')
content = content.replace('TransactionType::Sale', 'TransactionType::Sell')
content = content.replace('TransactionType::Production', 'TransactionType::Buy')

# Fix record_transaction calls - correct signature is (step, skill_id, transaction_type, amount, counterparty_id)
content = re.sub(
    r'person\.record_transaction\((\d+), (TransactionType::\w+), &(\w+\.id), ([\d.]+), ([\d.]+)\)',
    r'person.record_transaction(\1, \3.id.clone(), \2, \4, None)',
    content
)

# Remove fields that don't exist
content = re.sub(r'assert_eq!\(person\.total_purchases_value, [\d.]+\);', '// Field removed', content)
content = re.sub(r'assert_eq!\(person\.total_sales_value, [\d.]+\);', '// Field removed', content)

# Remove restore_checkpoint test (doesn't exist)
content = re.sub(
    r'#\[test\]\s+fn test_engine_checkpoint_save_and_restore\(\) \{[^}]*\}[^}]*\}[^}]*\}',
    '// Checkpoint test removed',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'#\[test\]\s+fn test_engine_checkpoint_with_invalid_path\(\) \{[^}]*\}',
    '// Checkpoint test removed',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'#\[test\]\s+fn test_engine_checkpoint_with_production_enabled\(\) \{[^}]*\}[^}]*\}[^}]*\}',
    '// Checkpoint test removed',
    content,
    flags=re.DOTALL
)

# Write back
with open('src/tests/final_push_tests.rs', 'w') as f:
    f.write(content)

print("Fixed tests file")
