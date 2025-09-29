#!/usr/bin/env python3
"""
Validate that lana.yml only contains configuration keys that are defined in lana.default.yml
"""

import sys
from pathlib import Path
from typing import Any, Dict, List, Set

# Check for required dependencies
try:
    import yaml
except ImportError:
    print("❌ Error: PyYAML is required but not installed.")
    print("Please install it with: pip install PyYAML")
    print("Or using your system package manager:")
    print("  - macOS: brew install pyyaml")
    print("  - Ubuntu/Debian: apt-get install python3-yaml")
    sys.exit(1)


def get_all_paths(data: Any, prefix: str = "") -> Set[str]:
    """
    Recursively extract all configuration paths from a nested dictionary/list structure.
    Returns a set of dot-notation paths like 'app.notification.email.port'
    """
    paths = set()
    
    if isinstance(data, dict):
        for key, value in data.items():
            current_path = f"{prefix}.{key}" if prefix else key
            paths.add(current_path)
            paths.update(get_all_paths(value, current_path))
    elif isinstance(data, list):
        # For lists, we don't add indexed paths, just mark that this path contains a list
        if prefix:
            paths.add(f"{prefix}[]")
    
    return paths


def load_yaml_file(file_path: Path) -> Dict[str, Any]:
    """Load and parse a YAML file using safe loader (no custom tags allowed)"""
    try:
        with open(file_path, 'r') as f:
            return yaml.safe_load(f) or {}
    except FileNotFoundError:
        print(f"Error: File {file_path} not found")
        sys.exit(1)
    except yaml.YAMLError as e:
        print(f"Error parsing YAML file {file_path}: {e}")
        print("This might be due to custom YAML tags. Both configuration files should use simple YAML values only.")
        sys.exit(1)


def check_for_custom_tags(file_path: Path) -> bool:
    """
    Check if a YAML file contains custom tags (like !Finite, !SomeTag, etc.)
    Returns True if custom tags are found, False otherwise
    """
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Look for YAML tags pattern: !TagName
        import re
        custom_tag_pattern = r'!\w+'
        matches = re.findall(custom_tag_pattern, content)
        
        if matches:
            print(f"❌ Custom YAML tags found in {file_path.name}:")
            for match in set(matches):  # Remove duplicates
                print(f"  - {match}")
            return True
        
        return False
    except Exception as e:
        print(f"Error checking for custom tags in {file_path}: {e}")
        return True  # Fail safe - assume there are custom tags if we can't check


def validate_config_structure(lana_config: Dict[str, Any], default_config: Dict[str, Any]) -> bool:
    """
    Validate that all paths in lana_config exist in default_config
    lana.yml can only override values that have defaults in lana.default.yml
    Returns True if valid, False otherwise
    """
    lana_paths = get_all_paths(lana_config)
    default_paths = get_all_paths(default_config)
    
    # Find paths that exist in lana.yml but not in lana.default.yml
    invalid_paths = lana_paths - default_paths
    
    if invalid_paths:
        print("❌ Validation failed!")
        print(f"\nThe following configuration paths exist in lana.yml but not in lana.default.yml:")
        for path in sorted(invalid_paths):
            print(f"  - {path}")
        print(f"\nlana.yml can only override values that have defaults defined in lana.default.yml.")
        return False
    else:
        print("✅ Validation passed!")
        print("All configuration paths in lana.yml are defined in lana.default.yml")
        return True


def main():
    """Main validation function"""
    print("🔍 Validating Lana configuration files...")
    print()
    
    # Get the script directory and find the config files
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent.parent
    
    lana_yml = repo_root / "bats" / "lana.yml"
    lana_default_yml = repo_root / "dev" / "lana.default.yml"
    
    print(f"Validating configuration files:")
    print(f"  lana.yml: {lana_yml}")
    print(f"  lana.default.yml: {lana_default_yml}")
    print()
    
    # Check for custom YAML tags in both files (neither should have any)
    print("Checking for custom YAML tags...")
    
    has_custom_tags_lana = check_for_custom_tags(lana_yml)
    has_custom_tags_default = check_for_custom_tags(lana_default_yml)
    
    if has_custom_tags_lana or has_custom_tags_default:
        print()
        print("❗ Custom tags validation failed!")
        print()
        print("Neither lana.yml nor lana.default.yml should contain custom YAML tags like !Finite.")
        print("Both configuration files should use simple YAML values only.")
        print("Please replace custom tagged values with simple values.")
        print()
        sys.exit(1)
    
    print("✅ No custom YAML tags found in either configuration file")
    print()
    
    # Load both configuration files using safe loader
    lana_config = load_yaml_file(lana_yml)
    default_config = load_yaml_file(lana_default_yml)
    
    # Validate the structure
    print("Validating configuration structure...")
    is_valid = validate_config_structure(lana_config, default_config)
    
    if is_valid:
        print()
        print("🎉 Configuration validation successful!")
    else:
        print(f"\n💡 Tip: Make sure all configuration keys in lana.yml are also defined in lana.default.yml")
        print()
        print("❗ Configuration validation failed!")
        print()
        print("To fix this, you have two options:")
        print("1. Remove the invalid keys from bats/lana.yml")
        print("2. Add the missing keys to dev/lana.default.yml with appropriate default values")
        print()
        sys.exit(1)


if __name__ == "__main__":
    main()
