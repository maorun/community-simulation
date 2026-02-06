import re

with open('src/tests/final_push_tests.rs', 'r') as f:
    content = f.read()

# Fix Person::new calls - need to import Strategy and Location
content = content.replace(
    'use crate::person::{Person, TransactionType};',
    'use crate::person::{Person, TransactionType, Strategy, Location};'
)

# Fix Person::new calls - requires Strategy and Location
content = re.sub(
    r'let mut person = Person::new\(1, 100\.0, skill1, vec!\[skill2\.id\.clone\(\)\]\);',
    'let mut person = Person::new(1, 100.0, vec![skill1.clone()], Strategy::Conservative, Location::new(0.0, 0.0));',
    content
)

# Fix total_trades field access
content = content.replace('result.total_trades > 0', 'result.trade_volume_statistics.total_trades > 0')
content = content.replace('result.total_trades >= 0', 'result.trade_volume_statistics.total_trades >= 0')

# Fix save_to_file calls - requires compress parameter
content = re.sub(
    r'result\.save_to_file\(([^)]+)\)\.unwrap\(\);',
    r'result.save_to_file(\1, false).unwrap();',
    content
)

with open('src/tests/final_push_tests.rs', 'w') as f:
    f.write(content)

print("Fixed final push tests")
