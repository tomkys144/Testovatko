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
            block(height: 2em)[#tiaoma.pdf417((student_username + "@" + str(counter(page).get().first() - 1)))],
          )
        ]
      }
    }
  )
  body
}

#let assignment_header(title, group, clazz, date, student_name, student_username) = {
  // Top Grid: Left side (Info), Right side (Student + QR)
  grid(
    columns: (1fr, auto),
    gutter: 1em,
    [
      #text(weight: "bold", size: 1.5em)[#title] \
      #v(0.5em)

      #text(weight: "bold")[Jméno:] #student_name\
      #text(weight: "bold")[Třída:] #clazz | #text(weight: "bold")[Skupina:] #group \
      #text(weight: "bold")[Datum:] #date \
      #text(weight: "bold")[Podpis:]
    ],
    align(top + right, block(height: 2em)[
      // Generate QR code encoding the student name
      #tiaoma.pdf417(student_username + "@0")
    ]),
  )
  line(length: 100%, stroke: 1pt)
  v(1cm)
}

#let open_question(points: 0, lines: 1, qnum: "1", body) = {
  block(breakable: false, width: 100%, {
    grid(
      columns: (1fr, auto),
      gutter: 1em,
      [
        #text(weight: "bold")[Otázka #qnum:]
        #body
        // Multiply em by the integer 'lines'
        #v(2em * lines)
      ],
      align(top, text(weight: "bold")[
        (#points bodů)
      ]),
    )
  })
}

#let multiple_choice_question(points: 0, qnum: "1", options: (), body) = {
  block(breakable: false, width: 100%, {
    grid(
      columns: (1fr, auto),
      gutter: 1em,
      [
        // Question Text
        #text(weight: "bold")[Otázka #qnum:]
        #body
        #v(0.5em)

        #set enum(numbering: "a)")
        #enum(..options)
      ],
      align(top, text(weight: "bold")[
        (#points bodů)
      ]),
    )
  })
}

#let add_markers_around_table(start_id: 0, body) = {
  let m_size = 0.8cm
  align(center, grid(
    columns: (1cm, auto, 1cm),

    // Read directly from disk, no sys.inputs!
    image("marker_" + str(start_id) + ".png", width: m_size),
    grid.cell()[],
    image("marker_" + str(start_id + 1) + ".png", width: m_size),

    grid.cell()[],
    body,
    grid.cell()[],

    image("marker_" + str(start_id + 2) + ".png", width: m_size),
    grid.cell()[],
    image("marker_" + str(start_id + 3) + ".png", width: m_size),
  ))
}

#let finish_exam(student_name, student_username) = {
  // If we are on an odd page, add one empty page to make the total even
  context {
    if calc.odd(counter(page).get().first()) {
      pagebreak()
      // Optional: Add text to indicate it's intentionally blank
      set page(
        margin: (top: 3cm),
        header: block(stroke: (bottom: 2pt), width: 100%, inset: (bottom: 5pt))[
          #grid(
            columns: (1fr, auto, 1fr),
            align: (left, center, right),
            text(weight: "bold", size: 12pt)[#student_name],
            block()[],
            block(height: 2em)[#tiaoma.pdf417(student_username + "@" + str(counter(page).get().first()))],
          )
        ]
      )
      str(counter(page).get().first())
      align(center + top)[_Prostor na poznámky._]
    }
  }
}
