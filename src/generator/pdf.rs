use anyhow::{Context, Result};
use lopdf::{Document as PdfDocument, Object, ObjectId};
use std::collections::BTreeMap;

pub fn merge_pdfs(input_paths: &[String], output_path: &str) -> Result<()> {
    let mut documents = Vec::new();
    for path in input_paths {
        documents.push(PdfDocument::load(path).with_context(|| format!("Loading {}", path))?);
    }

    let mut max_id = 1;
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut merged_doc = PdfDocument::with_version("1.5");

    for mut doc in documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        documents_pages.extend(
            doc.get_pages()
                .into_iter()
                .map(|(_, object_id)| (object_id, doc.get_object(object_id).unwrap().to_owned())),
        );
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects.iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object { id } else { *object_id },
                    object.clone(),
                ));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }
                    pages_object = Some((
                        if let Some((id, _)) = pages_object { id } else { *object_id },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                merged_doc.objects.insert(*object_id, object.clone());
            }
        }
    }

    let catalog_object = catalog_object.context("Catalog root not found")?;
    let pages_object = pages_object.context("Pages root not found")?;

    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.0);
            merged_doc.objects.insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages.into_iter().map(|(object_id, _)| Object::Reference(object_id)).collect::<Vec<_>>(),
        );
        merged_doc.objects.insert(pages_object.0, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines");
        merged_doc.objects.insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    merged_doc.trailer.set("Root", catalog_object.0);
    merged_doc.max_id = merged_doc.objects.len() as u32;

    merged_doc.renumber_objects();
    merged_doc.compress();
    merged_doc.save(output_path).context("Failed to save printable PDF")?;

    Ok(())
}