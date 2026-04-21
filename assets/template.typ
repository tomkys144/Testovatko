#import "@preview/tiaoma:0.3.0"

#let exam_setup(student_name, student_username, body) = {
    show raw: it => {
        if it.block {
            block(fill: luma(245), inset: 10pt, radius: 4pt, width: 100%, text(fill: black, it.text))
        } else {
            box(fill: luma(245), inset: (x: 3pt, y: 0pt), outset: (y: 3pt), radius: 2pt, text(fill: black, it.text))
        }
    }

    set page(
        margin: (top: 3cm),
        header: context {
            if counter(page).get().first() > 1 {
                block(stroke: (bottom: 2pt), width: 100%, inset: (bottom: 5pt))[
                    #grid(
                        columns: (1fr, auto, 1fr),
                        align: (left, center, right),
                        text(weight: "bold", size: 12pt)[#student_name],
                        block()[],
                        block(height: 2em)[#tiaoma.pdf417((
                                student_username + "@" + str(counter(page).get().first() - 1)
                            ))],
                    )
                ]
            }
        },
    )
    body
}

#let assignment_header(title, group, class, date, student_name, student_username) = {
    place(top + right)[
        #block(height: 2em)[#tiaoma.pdf417(student_username + "@0")]
    ]

    v(3em)

    align(center)[
        #text(weight: "bold", size: 1.5em)[#title] \
        #v(.5em)
        #text(weight: "semibold", size: 1.2em)[Třída: #class #h(2em) Skupina: #group] \
        #v(2em)
    ]

    block([
        #set text(size: 1.1em)

        #text(weight: "bold")[Jméno:]
        #box(width: 80%, stroke: none, outset: (bottom: 3pt))[#student_name] \
        #v(1em)
        #text(weight: "bold")[Datum:]
        #box(width: 40%, stroke: none, outset: (bottom: 3pt))[#date] \
        #v(1em)
        #text(weight: "bold")[Podpis:]
        #box(width: 20%, stroke: (bottom: 1pt), outset: (bottom: 3pt))[] \
    ])
    v(1cm)
}

#let open_question(points: 0, lines: 1, qnum: "1", body) = {
    block(
        breakable: false,
        width: 100%,
        {
            grid(
                columns: (1fr, auto),
                gutter: 1em,
                [
                    #text(weight: "bold")[Otázka #qnum:]
                    #body
                    #v(2em * lines)
                ],
                align(
                    top,
                    text(weight: "bold")[
                        (#points bodů)
                    ],
                ),
            )
        },
    )
}

#let multiple_choice_question(points: 0, qnum: "1", options: (), body) = {
    block(
        breakable: false,
        width: 100%,
        {
            grid(
                columns: (1fr, auto),
                gutter: 1em,
                [
                    #text(weight: "bold")[Otázka #qnum:]
                    #body
                    #v(0.5em)

                    #set enum(numbering: "a)")
                    #enum(..options)
                ],
                align(
                    top,
                    text(weight: "bold")[
                        (#points bodů)
                    ],
                ),
            )
        },
    )
}

#let add_markers_around_table(start_id: 0, body) = {
    let m_size = 0.8cm
    align(
        center,
        block(breakable: false)[
            #grid(
                columns: (1cm, auto, 1cm),
                image("marker_" + str(start_id) + ".png", width: m_size),
                grid.cell()[],
                image("marker_" + str(start_id + 1) + ".png", width: m_size),

                grid.cell()[],
                body,
                grid.cell()[],

                image("marker_" + str(start_id + 2) + ".png", width: m_size),
                grid.cell()[],
                image("marker_" + str(start_id + 3) + ".png", width: m_size),
            )
        ],
    )
}

#let finish_exam(student_name, student_username) = context [
    #let p = here().page()
    #if calc.odd(p) [
        #h(0pt)
        #pagebreak(weak: false)
        #align(center + top)[_Prostor na poznámky._]
    ]
]
