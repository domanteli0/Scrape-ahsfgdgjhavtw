use reqwest::{blocking, Url};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate, Text};
use std::{env};
use thiserror::{self, Error};
use lazy_regex::regex;

#[derive(Debug)]
struct Item {
    question: String,
    image: Option<Url>, // Should be URL
    possible: Vec<String>,
    correct: Vec<String>,
    explanation: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    let url = args
        .get(1)
        .expect("Expecten an URL as the first argument");

    let body = blocking::get(url)?.text()?;
    println!("Got body");

    let document = Document::from(body.as_ref());

    // println!("body = {:?}", body);
    // println!("document = {:?}", document);

    let mut nodes = document
            // .find(Class("thecontent"))
            .find(Attr("class", "thecontent clearfix"))
            .take(1).last().expect("Cound not find content")
            .children()
            .skip_while(|c| 
                c.name() != Some("h3")
            )
            .skip(1)
            .filter(|n| n.text() != "\n")
            .collect::<Vec<_>>()
            ;
    
    if nodes.get(0).expect("nu, krč čia turbūt dalykų").name() == Some("h4") {
        nodes = nodes[1..].into();
    }

    let mut nodes = nodes.into_iter();
    let questions = get_questions(&mut nodes);

    // println!("{:#?}", &questions[..]);
    // println!("No of preparsed: {}", questions.len());

    // println!("No of parsed: {}",
    //     questions.into_iter()
    //         .filter_map(|q| TryInto::<Item>::try_into(q).ok())
    //         .count()
    // );


    // println!("{questions:#?}");
    // println!("{:#?}",
    //     questions.into_iter()
    //         .map(|q| TryInto::<Item>::try_into(q))
    //         .filter(|q| q.is_err())
    //         .collect::<Vec<_>>()
    // );

    for quest in questions.into_iter() {
        let question: Result<Item, _> = quest.try_into();

        println!("{:#?}", question.map(|q| q.question));
    }

    Ok(())
}

impl<'a> TryFrom<Vec<Node<'a>>> for Item {
    type Error = String;

    fn try_from(value: Vec<Node>) -> Result<Self, Self::Error> {
        let regex = regex!(r"^[0-9]*\..Match");

        let question = value
            .iter()
            .find(|n| n.name() == Some( "p") )
            .ok_or("Could not find question headline")?;

        let question = question
            .children()
            .last()
            .ok_or("could not find `strong` el")?
            .text()
            ;

        let answers = {
            if regex.is_match(&question) {
                Vec::new()
            } else {
                value
                    .iter()
                    .find(|n| n.is(Name("ul")))
                    .ok_or("Coulnd not find possible answers")?
                    .children()
                    .filter(|n| n.text() != "\n")
                    .collect::<Vec<_>>()
            }
       };

        let image = value.iter()
            .map(|n| n.descendants())
            .flatten()
            .filter(|n| n.is(Name("img")))
            .last()
            .and_then(|img| img.attr("src"));

        let image = {
            if image.is_some() {
                Some(
                Url::parse(image.unwrap())
                    .map_err(|_| "Image found but src attr could not be parsed to a valid url")?
                )
            }
            else { None }
        };

        let explanation = value.iter()
            .find(|n| n.is(Name("div")))
            .and_then(|n| n.children().find(|nn| nn.is(Name("p"))))
            .map(|n| n.text())
            .filter(|n| n.starts_with("Explanation: "))
            .map(|ex| ex["Explanation: ".len()..].to_owned())
            ;

        // println!("Explanation {:#?}", explanation);
       
        Ok( Item {
            question,
            image,
            possible: answers.clone()
                .into_iter()
                .map(|n| n.text())
                .collect(),
            correct: answers
                .into_iter()
                .filter_map(|n| n.children().last())
                .filter_map(|n| n.children().last())
                .map(|n| n.text())
                .collect::<Vec<_>>(),
            explanation,

        })
    }
}

fn get_questions<'a, T>(iter: &mut T) -> Vec<Vec<Node<'a>>>
where T: Iterator<Item = Node<'a>>
{
    let mut ret = Vec::new();
    let mut question = Vec::new();
    let regex = regex!(r"^[0-9]*\.");

    let node = {
        let temp = iter.next();
        if temp.is_none() { return ret; }
        temp.unwrap()
    };
    question.push(node);

    while let Some(node) = iter.next() {
        if regex.is_match(node.text().as_ref())
            // && ( node.children().last().into_iter().any( |n| n.name() == Some("strong") ) || node.children().last().into_iter().any(|n| n.is(Name("b"))))
        {
            ret.push(question);
            question = Vec::new();
        }

        question.push(node);
    }

    ret
}
