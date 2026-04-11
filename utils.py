import json
import argparse
import sys


def get_valid_int(prompt_text):
    while True:
        try:
            user_input = input(prompt_text).strip()
            num = int(user_input) if user_input else 0
            if num >= 0:
                return num
            print("Please enter a positive number.")
        except ValueError:
            print("Invalid input. Please enter a number.")


def create_empty():
    print("\n--- Empty Students Template Setup ---")
    num_students = get_valid_int("How many empty students do you want to create? (Enter a number): ")

    empty_students = [{"name": "", "username": ""} for _ in range(num_students)]

    with open("students.json", "w", encoding="utf-8") as f:
        json.dump(empty_students, f, indent=2, ensure_ascii=False)
    print(f"Created 'students.json' with {num_students} empty records.")

    print("\n--- Empty Quiz Template Setup ---")
    num_sections = get_valid_int("How many sections should the quiz have? (Enter a number): ")

    empty_quiz = {
        "title": "",
        "group": "",
        "class": "",
        "date": "",
        "sections": []
    }

    for i in range(num_sections):
        num_questions = get_valid_int(f"  How many empty questions should section {i + 1} have? (number): ")

        section = {
            "title": "",
            "type": "",
            "description": "",
            "questions": []
        }

        for _ in range(num_questions):
            section["questions"].append({
                "question": "",
                "points": 0,
                "lines": 0,
                "options": []
            })

        empty_quiz["sections"].append(section)

    with open("quiz.json", "w", encoding="utf-8") as f:
        json.dump(empty_quiz, f, indent=2, ensure_ascii=False)

    print(f"Created 'quiz.json' with {num_sections} empty sections.")


def interactive_students():
    students = []
    print("\n--- Interactive Students Setup ---")
    print("Leave the name blank and press Enter to finish adding.")

    while True:
        name = input("Student Name: ").strip()
        if not name:
            break
        username = input("Username (e.g., john.doe): ").strip()
        students.append({"name": name, "username": username})
        print(f"Added {name}.")

    with open("students.json", "w", encoding="utf-8") as f:
        json.dump(students, f, indent=2, ensure_ascii=False)
    print(f"\nSaved {len(students)} students to 'students.json'.")


def interactive_quiz():
    print("\n--- Interactive Quiz Setup ---")
    quiz = {
        "title": input("Quiz Title: ").strip(),
        "group": input("Group: ").strip(),
        "class": input("Class: ").strip(),
        "date": input("Date (YYYY-MM-DD): ").strip(),
        "sections": []
    }

    while True:
        add_section = input("\nAdd a new section? (y/n): ").strip().lower()
        if add_section != 'y':
            break

        section = {
            "title": input("Section Title: ").strip(),
            "type": input("Type ('open' or 'multiple_choice'): ").strip(),
            "description": input("Description: ").strip(),
            "questions": []
        }

        while True:
            add_q = input(f"\nAdd a question to '{section['title']}'? (y/n): ").strip().lower()
            if add_q != 'y':
                break

            question = {
                "question": input("Question text: ").strip(),
                "points": int(get_valid_int("Points (number): "))
            }

            if section["type"] == "open":
                question["lines"] = get_valid_int("Number of blank lines for answer: ")
            elif section["type"] == "multiple_choice":
                question["options"] = []
                while True:
                    add_opt = input("Add an option? (y/n): ").strip().lower()
                    if add_opt != 'y':
                        break
                    opt_text = input("Option text: ").strip()
                    is_correct = input("Is this option correct? (y/n): ").strip().lower() == 'y'
                    question["options"].append({"text": opt_text, "is_correct": is_correct})

            section["questions"].append(question)

        quiz["sections"].append(section)

    with open("quiz.json", "w", encoding="utf-8") as f:
        json.dump(quiz, f, indent=2, ensure_ascii=False)
    print("\nSaved 'quiz.json' successfully.")


if __name__ == "__main__":
    try:
        parser = argparse.ArgumentParser()
        parser.add_argument("mode", choices=["empty", "interactive"])

        args = parser.parse_args()

        if args.mode == "empty":
            create_empty()
        elif args.mode == "interactive":
            interactive_students()
            interactive_quiz()
    except KeyboardInterrupt:
        print("\nProcess interrupted. Exiting.")
        sys.exit(0)