import sys
import xml.etree.ElementTree as ET


def verify_parent_pointers(xml_path):
    """
    Verifies that all parent pointers point to classes in the same config
    """

    tree = ET.parse(xml_path)
    root = tree.getroot()

    errors = []

    for course in root.findall(".//course"):
        course_id = course.get("id")

        for config in course.findall("./config"):
            config_id = config.get("id")

            classes = config.findall(".//class")
            class_ids = {cls.get("id") for cls in classes}

            for cls in classes:
                cls_id = cls.get("id")
                parent_id = cls.get("parent")

                if parent_id is not None:
                    if parent_id not in class_ids:
                        errors.append(
                            f"ERROR: Class {cls_id} in course {course_id}, "
                            f"config {config_id} has parent {parent_id} "
                            f"which is NOT in the same config."
                        )

    return errors


def main():
    if len(sys.argv) != 2:
        print("Usage: python parent_same_config_theorem.py <problem.xml>")
        sys.exit(1)

    xml_path = sys.argv[1]
    errors = verify_parent_pointers(xml_path)

    if not errors:
        print("All parent pointers are valid.")
    else:
        print("Invalid parent pointers found:\n")
        for err in errors:
            print(err)
        sys.exit(2)


if __name__ == "__main__":
    main()
