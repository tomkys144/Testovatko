import json
import string

import cv2
import easyocr
import numpy as np
from pdf2image import convert_from_path
import zxingcpp

from PIL import Image

ARUCO_DICT = cv2.aruco.getPredefinedDictionary(cv2.aruco.DICT_4X4_50)
PARAMETERS = cv2.aruco.DetectorParameters()
CHOICE_IDS = [4, 5, 6, 7]
RESULTS_IDS = [0, 1, 2, 3]

reader = easyocr.Reader(['en'], gpu=False)


def get_box(page, corners):
    points = np.hstack((np.min(corners, axis=1), np.max(corners, axis=1)))
    inner_x_min = int(np.max(np.partition(points[:, 2], 2)[:2]))
    inner_x_max = int(np.min(np.partition(points[:, 0], 2)[2:]))
    inner_y_min = int(np.max(np.partition(points[:, 3], 2)[:2]))
    inner_y_max = int(np.min(np.partition(points[:, 1], 2)[2:]))

    return [
        page[inner_y_min:inner_y_max, inner_x_min:inner_x_max],
        [inner_x_min, inner_y_min, inner_x_max, inner_y_max]
    ]


def find_cells(img):
    kernel_length_v = (np.array(img).shape[1]) // 120
    vertical_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, kernel_length_v))
    im_temp1 = cv2.erode(img, vertical_kernel, iterations=3)
    vertical_lines_img = cv2.dilate(im_temp1, vertical_kernel, iterations=3)

    kernel_length_h = (np.array(img).shape[1]) // 40
    horizontal_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (kernel_length_h, 1))
    im_temp2 = cv2.erode(img, horizontal_kernel, iterations=3)
    horizontal_lines_img = cv2.dilate(im_temp2, horizontal_kernel, iterations=3)

    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (3, 3))
    table_segment = cv2.addWeighted(vertical_lines_img, 0.5, horizontal_lines_img, 0.5, 0.0)
    table_segment = cv2.erode(cv2.bitwise_not(table_segment), kernel, iterations=2)
    thresh, table_segment = cv2.threshold(table_segment, 0, 255, cv2.THRESH_OTSU)

    contours, hierarchy = cv2.findContours(table_segment, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

    return contours


def sort_contours_grid(contours, threshold=15):
    cnts_with_boxes = []
    for c in contours:
        cnts_with_boxes.append([c, cv2.boundingRect(c)])

    cnts_with_boxes.sort(key=lambda b: b[1][0])

    sorted_contours = []
    current_col = []

    previous_x = -threshold - 1

    if cnts_with_boxes:
        previous_x = cnts_with_boxes[0][1][0]

    for c, box in cnts_with_boxes:
        x, y, w, h = box

        if abs(x - previous_x) <= threshold:
            current_col.append((c, box))
        else:
            current_col.sort(key=lambda b: b[1][1])
            sorted_contours.extend([item[0] for item in current_col])
            current_col = [(c, box)]
            previous_x = x

    if current_col:
        current_col.sort(key=lambda b: b[1][1])
        sorted_contours.extend([item[0] for item in current_col])

    return sorted_contours


def filter_contours_by_size(contours, tolerance=0.2, min_group_size=3):
    if not contours:
        return []

    cnt_data = []
    for c in contours:
        _, _, w, h = cv2.boundingRect(c)
        area = w * h
        if area > 50:
            cnt_data.append({'cnt': c, 'area': area, 'w': w, 'h': h})

    if not cnt_data:
        return []

    cnt_data.sort(key=lambda b: b['area'])

    groups = []
    if cnt_data:
        current_group = [cnt_data[0]]
        for i in range(1, len(cnt_data)):
            ref_area = current_group[0]['area']
            current_area = cnt_data[i]['area']

            if abs(current_area - ref_area) / ref_area <= tolerance:
                current_group.append(cnt_data[i])
            else:
                groups.append(current_group)
                current_group = [cnt_data[i]]

            # Append the last group
        groups.append(current_group)

    final_contours = []
    for g in groups:
        if len(g) >= min_group_size:
            for item in g:
                final_contours.append(item['cnt'])

    if not final_contours:
        print(f"Warning: No groups of {min_group_size}+ similar cells found.")
        return []

    return final_contours


def correct_multichoice(page, corners, answers, points):
    box, coords = get_box(page, corners)

    gray = cv2.cvtColor(box, cv2.COLOR_RGB2GRAY)
    blurred = cv2.GaussianBlur(gray, (5, 5), 0)
    mask = cv2.threshold(blurred, 0, 255, cv2.THRESH_BINARY_INV | cv2.THRESH_OTSU)[1]

    contours = find_cells(mask)
    contours = filter_contours_by_size(contours, tolerance=0.2, min_group_size=len(answers) * 3)
    contours = sort_contours_grid(contours, 10)

    selected = []
    first_iter = True
    lasty = -1
    dots = []
    rowidx = 0
    colidx = -1
    for c in contours:
        x, y, w, h = cv2.boundingRect(c)
        debug_img = cv2.cvtColor(mask, cv2.COLOR_GRAY2BGR)
        cv2.drawContours(debug_img, [c], -1, (0, 255, 0), 3)

        if first_iter == True:
            first_iter = False
            radius = (w + h) / 8
            continue

        cell = mask[y:y + h, x:x + w]

        if y < lasty:
            lasty = y
            colidx += 1
            rowidx = 0
            if len(selected) < colidx:
                selected.append('')
            continue

        lasty = y

        if colidx < 0:
            continue

        cnts, _ = cv2.findContours(cell, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        total_perimeter = 0
        for c in cnts:
            perimeter = cv2.arcLength(c, True)
            total_perimeter += perimeter

        if (total_perimeter > 1.5 * cell.shape[0]) and (total_perimeter < cell.shape[1] * 6):
            if len(selected) == colidx:
                selected.append(string.ascii_uppercase[rowidx])
                dots.append([[x + w / 2, y + h / 2]])
            else:
                selected[colidx] = ''
                dots[colidx].append([x + w / 2, y + h / 2])

        rowidx += 1

    if len(selected) != len(answers):
        selected.append('')

    is_correct = np.array(answers) == np.array(selected)
    for idx, dot in enumerate(dots):
        for coord in dot:
            if not radius: radius = 10
            center = (int(coord[0] + coords[0]), int(coord[1] + coords[1]))

            if is_correct[idx]:
                cv2.circle(page, center, int(radius), (0, 255, 0), -1)
            else:
                cv2.circle(page, center, int(radius), (255, 0, 0), -1)

    points = np.array(points)[is_correct].sum()

    return points, page


def calculate_points(page, corners, section_points):
    box, coords = get_box(page, corners[0])

    gray = cv2.cvtColor(box, cv2.COLOR_RGB2GRAY)
    blurred = cv2.GaussianBlur(gray, (5, 5), 0)
    mask = cv2.threshold(blurred, 0, 255, cv2.THRESH_BINARY_INV | cv2.THRESH_OTSU)[1]

    contours = find_cells(mask)
    contours = filter_contours_by_size(contours, tolerance=0.01, min_group_size=len(section_points) + 1)
    contours = sort_contours_grid(contours, 10)

    first_iter = True
    lasty = len(page)
    total_points = 0
    rowidx = -1
    colidx = -1

    for c in contours:
        x, y, w, h = cv2.boundingRect(c)
        debug_img = cv2.cvtColor(mask, cv2.COLOR_GRAY2BGR)
        cv2.drawContours(debug_img, [c], -1, (0, 255, 0), 3)

        if h < mask.shape[0] / 4: continue
        if w < (mask.shape[1] / len(section_points) + 1) / 2: continue

        if (rowidx >= 2) or (rowidx < 0):
            rowidx = 0
            colidx += 1
            continue

        if rowidx == 1:
            cell = mask[y:y + h, x:x + w]
            results = reader.readtext(cell)
            pts = ''
            for (bbox, text, prob) in results:
                if prob > 0.5:
                    pts = text

            if pts:
                total_points += int(pts.replace(" ", ""))
                section_points[str(colidx)] = pts

            if not pts:
                if colidx >= len(section_points):
                    (text_w, text_h), baseline = cv2.getTextSize(str(total_points),
                                                                 cv2.FONT_HERSHEY_SIMPLEX, 1, 2)
                    xc = int(x + w // 2 + coords[0])
                    yc = int(y + h // 2 + coords[1])
                    xt = xc - (text_w // 2)
                    yt = yc + (text_h // 2)
                    cv2.putText(page, str(total_points), (xt, yt), cv2.FONT_HERSHEY_SIMPLEX, 1,
                                (255, 0, 0), 2, cv2.LINE_AA)
                    print("written mark")
                else:
                    if section_points[str(colidx)] >= 0:
                        pts = section_points[str(colidx)]
                        total_points += pts
                        (text_w, text_h), baseline = cv2.getTextSize(str(pts),
                                                                     cv2.FONT_HERSHEY_SIMPLEX, 1, 2)
                        xc = int(x + w // 2 + coords[0])
                        yc = int(y + h // 2 + coords[1])
                        xt = xc - (text_w // 2)
                        yt = yc + (text_h // 2)
                        cv2.putText(page, str(pts), (xt, yt), cv2.FONT_HERSHEY_SIMPLEX, 1,
                                    (255, 0, 0), 2, cv2.LINE_AA)
                        print("written mark")
        rowidx += 1

    return page, total_points, section_points

def process_test(test_pages: list,
                 results_corners: list[np.ndarray],
                 choice_corners: list[np.ndarray],
                 choice_page: list[int],
                 student_id: str,
                 answers: dict,
                 quiz: dict
                 ):
    choice_id = 0

    section_points = {}

    for section, ans in answers.items():
        if not ans:
            section_points[section] = -1
            continue
        questions = quiz['sections'][int(section)]['questions']
        pts = [q['points'] for q in questions]
        points, page = correct_multichoice(test_pages[choice_page[choice_id]], choice_corners[choice_id], ans, pts)
        test_pages[choice_page[choice_id]] = page
        section_points[section] = int(points)
        choice_id += 1


    page, total_points, section_points = calculate_points(test_pages[0], results_corners, section_points)
    test_pages[0] = page


    if test_pages:
        pil_pages = []
        for page in test_pages:
            # Convert Numpy Array -> PIL Image
            # Note: Assuming 'page' is already in RGB format from pdf2image
            pil_pages.append(Image.fromarray(page))

        output_filename = f"res/marked_{student_id}.pdf"

        # Save the first image and append the rest
        pil_pages[0].save(
            output_filename,
            "PDF",
            resolution=100.0,
            save_all=True,
            append_images=pil_pages[1:]
        )
        print(f"Saved marked test: {output_filename}")

    return {"total_points": int(total_points), "section_points": section_points}


def mark(pdf_path, student_path, quiz_path, answer_path):
    quiz = json.load(open(quiz_path))
    answers = json.load(open(answer_path))
    students = json.load(open(student_path))

    images = convert_from_path(pdf_path)

    student_results = {}
    test_pages = []
    results_corners = []
    choice_corners = []
    student_id = ""
    choice_page = []
    ans = {}

    print(f"Processing {len(images)} images")
    test_page_num = 0

    for page_num, image in enumerate(images):
        img = np.array(image)
        img_cv = cv2.cvtColor(img, cv2.COLOR_RGB2BGR)

        detected_id = None
        barcodes = zxingcpp.read_barcodes(img_cv)
        for barcode in barcodes:
            if barcode.format == zxingcpp.BarcodeFormat.PDF417:
                detected_id = barcode.text
                break

        if detected_id and detected_id != student_id:
            if student_id:
                res = process_test(test_pages, results_corners, choice_corners, choice_page, student_id, ans, quiz)
                student_results[student_id] = res
                print(f"Finished {student_id}, switching to {detected_id}")

            student_id = detected_id
            test_pages.clear()
            results_corners.clear()
            choice_corners.clear()
            ans = answers[student_id]
            choice_page.clear()
            test_page_num = 0

            test_pages.append(img)

        elif student_id:
            test_pages.append(img)
            test_page_num += 1
        else:
            test_pages.append(img)

        aruco_detector = cv2.aruco.ArucoDetector(ARUCO_DICT, PARAMETERS)
        corners, ids, rejected = aruco_detector.detectMarkers(img_cv)

        if ids is None:
            continue

        if np.any(np.isin(ids, RESULTS_IDS)):
            corners_idx = np.where(np.isin(ids.flatten(), RESULTS_IDS))[0]
            results_corners.append(np.array(corners)[corners_idx].squeeze())

        if np.any(np.isin(ids, CHOICE_IDS)):
            choice_idx = np.where(np.isin(ids.flatten(), CHOICE_IDS))[0]
            choice_corners.append(np.array(corners)[choice_idx].squeeze())
            choice_page.append(test_page_num)

    if student_id:
        res = process_test(test_pages, results_corners, choice_corners, choice_page, student_id, ans, quiz)
        student_results[student_id] = res
        print(f"Finished final student: {student_id}")

    json.dump(student_results, open("res/marked_students.json", "w"))

if __name__ == "__main__":
    pdf_path = "./quiz-filled.pdf"
    student_path = "students.json"
    quiz_path = "quiz.json"
    answer_path = "quiz_ans.json"

    mark(pdf_path, student_path, quiz_path, answer_path)
