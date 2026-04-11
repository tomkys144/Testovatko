import argparse
import sys

def main():
    parser = argparse.ArgumentParser(
        description="Test generator and automatic marking tool.",
        epilog="Use 'python main.py <command> -h' for help with a specific command."
    )

    subparsers = parser.add_subparsers(dest="command", required=True, help="Available commands")

    parser_gen = subparsers.add_parser("generate", help="Generate PDF tests for students.")
    parser_gen.add_argument("--quiz", default="quiz.json",
                            help="Path to the assignment JSON file (default: quiz.json)")
    parser_gen.add_argument("--students", default="students.json",
                            help="Path to the students JSON file (default: students.json)")
    parser_gen.add_argument("--output", default="quiz.pdf",
                            help="Output merged PDF with tests (default: quiz.pdf)")
    parser_gen.add_argument("--answers", default="quiz_ans.json",
                            help="Output JSON with correct answers (default: quiz_ans.json)")

    parser_mark = subparsers.add_parser("mark", help="Mark scanned tests.")
    parser_mark.add_argument("--pdf", default="quiz-filled.pdf",
                             help="Path to the scanned PDF with tests (default: quiz-filled.pdf)")
    parser_mark.add_argument("--students", default="students.json",
                             help="Path to the students JSON file (default: students.json)")
    parser_mark.add_argument("--quiz", default="quiz.json",
                             help="Path to the assignment JSON file (default: quiz.json)")
    parser_mark.add_argument("--answers", default="quiz_ans.json",
                             help="Path to the JSON with correct answers (default: quiz_ans.json)")

    parser_setup = subparsers.add_parser("setup", help="Utility scripts to generate configuration files.")
    parser_setup.add_argument("mode", choices=["empty", "interactive"],
                              help="Choose 'empty' for templates or 'interactive' for step-by-step setup.")

    args = parser.parse_args()

    if args.command == "generate":
        import generate # Lazy import
        print("Starting test generation...")
        generate.generate(
            quiz_path=args.quiz,
            student_path=args.students,
            output_path=args.output,
            answers_path=args.answers
        )
        print("Generation completed successfully.")

    elif args.command == "mark":
        import mark # Lazy import: easyocr will ONLY load if this command is run
        print("Starting test marking...")
        mark.mark(
            pdf_path=args.pdf,
            student_path=args.students,
            quiz_path=args.quiz,
            answer_path=args.answers
        )
        print("Marking completed successfully.")

    elif args.command == "setup":
        import utils
        if args.mode == "empty":
            utils.create_empty()
        elif args.mode == "interactive":
            utils.interactive_students()
            utils.interactive_quiz()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nProcess interrupted. Exiting.")
        sys.exit(0)