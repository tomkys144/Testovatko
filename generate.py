import json
from pathlib import Path
from random import shuffle
import cv2
import typst
import string
import os
import glob

from pypdf import PdfWriter

def generate_aruco_markers():
    """Generates 8 markers (0-7) to support up to 2 tables."""
    dictionary = cv2.aruco.getPredefinedDictionary(cv2.aruco.DICT_4X4_50)
    print("Generating ArUco markers...\n")
    for i in range(8):
        # Generate 200x200 marker
        img = cv2.aruco.generateImageMarker(dictionary, i, 200)
        # Add whitespace border so black marker doesn't merge with table lines
        img = cv2.copyMakeBorder(img, 20, 20, 20, 20, cv2.BORDER_CONSTANT, value=255)
        filename = f"marker_{i}.png"
        cv2.imwrite(filename, img)

def parse_assignment(path: str):
    with open(path) as f:
        quiz = json.load(f)
        f.close()

    total_points = 0
    for section in quiz['sections']:
        for question in section['questions']:
            total_points += question['points']
    quiz['total_points'] = total_points
    return quiz

def get_template_content(path: str):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return f.read()
    except FileNotFoundError:
        return ""


def make_answer_grid(section, section_number):
    """
    Creates a grid of answer boxes (1: A B C D, 2: A B C D...) for the section.
    """
    num_questions = len(section['questions'])
    num_answers = 0

    for i in range (num_questions):
        question_answers = len(section['questions'][i]['options'])
        if question_answers > num_answers: num_answers = question_answers

    tbl = "#table(\n"
    tbl += f"columns: (2.5em, {', '.join(['2.5em' for _ in range(num_questions)])}),\n"
    tbl += f"rows: 2.5em,\n"
    tbl += f'table.cell()[], {', '.join([f'text(weight: "bold", size: 1.8em)[{section_number}.{j}]' for j in range(num_questions)])},\n'
    for i in range(num_answers):
        tbl += f'text(weight: "bold", size: 1.8em)[{string.ascii_uppercase[i]}], {', '.join(['table.cell()[]' for _ in range(num_questions)])},\n'
    tbl += ")"
    return f'#add_markers_around_table(start_id: {4})[\n{tbl}\n]\n'

def make_section(section: dict, section_number: int):
    tpst = f"= {section['title']}\n"
    question_number = 1
    answers = []
    if section['type'] == 'open':
        for question_number,question in enumerate(section['questions']):
            tpst += f'#open_question(points: {question['points']}, lines: {question['lines']}, qnum: "{section_number}.{question_number+1}")[{question['question']}]\n\n'

    elif section['type'] == 'multiple_choice':
        for question_number,question in enumerate(section['questions']):
            options = question['options']
            shuffle(options)
            opts = []
            for idx, option in enumerate(options):
                opts.append(f'"{option['text']}"')
                if option['is_correct']: answers.append(string.ascii_uppercase[idx])
            tpst += f'#multiple_choice_question(points: {question['points']}, qnum: "{section_number}.{question_number+1}", options: {"(" + ", ".join(opts) + ")"})[{question['question']}]\n\n'

        tpst += make_answer_grid(section, section_number)
    return tpst, answers

def make_header(quiz, student):
    return f'#assignment_header("{quiz["title"]}", "{quiz["group"]}", "{quiz["class"]}", "{quiz["date"]}", "{student['name']}", "{student['username']}")\n'


def make_points_summary(quiz, start_marker_id=0):
    section_points = []
    for section in quiz['sections']:
        pts = sum(q['points'] for q in section['questions'])
        section_points.append(pts)

    total_points = sum(section_points)
    num_sections = len(section_points)

    tbl = ""
    tbl += "#table(\n"
    tbl += f"  columns: ({', '.join(['5em'] * (num_sections + 1))}),\n"
    tbl += "  align: center,\n"
    tbl += "  inset: 8pt,\n"

    headers = [f'text(weight: "bold")[{i + 1}]' for i in range(num_sections)]
    headers.append(r"$Sigma$")
    tbl += "  " + ", ".join(headers) + ",\n"

    points_row = [f'text[{str(p)}]' for p in section_points]
    points_row.append(f'text(weight: "bold")[{total_points}]')
    tbl += "  " + ", ".join(points_row) + ",\n"

    empty_row = ["v(1.5em)"] * (num_sections + 1)
    tbl += "  " + ", ".join(empty_row) + "\n"

    tbl += ")"
    return f'#add_markers_around_table(start_id: {start_marker_id})[\n{tbl}\n]\n'

def make_quiz(quiz: dict, output_path: str, student: dict, typst_path:str= ""):
    if not typst_path:
        typst_path = "tmp.typ"

    Path(typst_path).touch(exist_ok=True)

    answers = {}

    with open(typst_path, '+w') as f:
        f.write('#import "template.typ":*\n\n')
        f.write('#set heading(numbering: "1.1.")\n\n')
        f.write(f'#show: doc => exam_setup("{student["name"]}", "{student['username']}", doc)\n\n')
        f.write(make_header(quiz, student))
        f.write(make_points_summary(quiz, start_marker_id=0))

        for section_number,section in enumerate(quiz['sections']):
            tpst, ans = make_section(section,section_number + 1)
            f.write(tpst)

            answers[section_number] = ans

        f.write(f'\n#finish_exam("{student["name"]}", "{student['username']}")\n')

        f.close()

    typst.compile(typst_path, output=output_path)
    return answers

def generate(quiz_path: str, student_path: str, output_path: str, answers_path: str):
    generate_aruco_markers()
    quiz = parse_assignment(quiz_path)
    students = json.load(open(student_path))
    print("Generating individual quizes\n")

    merger = PdfWriter()
    answers = {}
    for student in students:
        print(f"Generating quiz {student['username']}\n")
        fname = f"tmp-{student['username']}.pdf"

        ans = make_quiz(quiz, fname, student)
        merger.append(fname)
        answers[student['username']] = ans

    merger.write(output_path)
    merger.close()

    os.remove("tmp.typ")
    for f in glob.glob("marker_*.png"):
        os.remove(f)

    for f in glob.glob("tmp*.pdf"):
        os.remove(f)

    json.dump(answers, open(answers_path, 'w+'))

if __name__ == '__main__':
    generate("quiz.json", "students.json", "quiz.pdf", "quiz_ans.json")
