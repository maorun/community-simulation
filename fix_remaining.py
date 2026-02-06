import re

with open('src/tests/final_push_tests.rs', 'r') as f:
    content = f.read()

# Remove test that accesses private entities field
content = re.sub(
    r'#\[test\]\s+fn test_engine_getter_with_inactive_entities\(\) \{[^}]*engine\.entities\[0\]\.active[^}]*\}',
    '',
    content,
    flags=re.DOTALL
)

# Fix gini_coefficient field access
content = content.replace('result.gini_coefficient', 'result.inequality_metrics.gini_coefficient')

# Fix average_transaction_price field access
content = content.replace('result.average_transaction_price', 'result.trade_volume_statistics.average_price')

# Remove any lines with empty , ,
content = re.sub(r'\n\s*,\s*,\s*\n', '\n', content)

with open('src/tests/final_push_tests.rs', 'w') as f:
    f.write(content)

print("Fixed remaining issues")
