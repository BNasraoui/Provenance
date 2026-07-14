use super::super::render_requirement;
use super::super::{render_search, render_topics};
use super::fixtures::{corpus_fixture, link};
use crate::wiki::model::{
    PageKind, SearchEntry, SearchIndexPage, Topic, TopicGroup, TopicIndexPage,
};

#[test]
fn topic_index_groups_real_links_and_describes_empty_domains() {
    let mut page = corpus_fixture().topics;
    page.groups.push(TopicGroup {
        topic: Topic::Defined {
            id: "domain_empty".to_string(),
            name: "Empty domain".to_string(),
            description: None,
        },
        requirements: vec![],
        rules: vec![],
    });
    let html = render_topics("default", &page);

    assert!(html.contains("id=\"domain-dom_invoicing\""), "{html}");
    assert!(
        html.contains("href=\"/requirements/req_saveinvoice_split/\""),
        "{html}"
    );
    assert!(html.contains("href=\"/rules/rule_sah_inv_016/\""), "{html}");
    assert!(
        html.contains("No requirements or rules are assigned to this domain."),
        "{html}"
    );
}

#[test]
fn topic_index_is_honest_when_no_domain_data_exists() {
    let page = TopicIndexPage {
        scope: "default".to_string(),
        title: "Topics by domain".to_string(),
        groups: vec![],
    };
    let html = render_topics("default", &page);
    assert!(html.contains("No domains, requirements, or rules are recorded"));
}

#[test]
fn search_page_contains_offline_index_and_safe_filtering_hooks() {
    let page = SearchIndexPage {
        scope: "default".to_string(),
        title: "Search".to_string(),
        entries: vec![SearchEntry {
            link: link(
                PageKind::Rule,
                "rule_safe",
                "<script>alert('title')</script>",
            ),
            kind: PageKind::Rule,
            statement: "unsafe \"statement\" & <tag>".to_string(),
        }],
    };
    let html = render_search("default", &page);

    assert!(html.contains("id=\"wiki-search\""), "{html}");
    assert!(html.contains("data-search-title="), "{html}");
    assert!(html.contains("data-search-statement="), "{html}");
    assert!(
        !html.contains("<li hidden data-search-entry"),
        "the unfiltered index should remain readable without JavaScript"
    );
    assert!(html.contains("href=\"/rules/rule_safe/\""), "{html}");
    assert!(!html.contains("<script>alert('title')</script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert"), "{html}");
    assert!(html.contains("unsafe &quot;statement&quot; &amp; &lt;tag&gt;"));
    assert!(
        !html.contains("innerHTML"),
        "search must not inject result HTML"
    );
}

#[test]
fn empty_search_index_explains_that_there_is_nothing_to_search() {
    let page = SearchIndexPage {
        scope: "default".to_string(),
        title: "Search".to_string(),
        entries: vec![],
    };
    let html = render_search("default", &page);
    assert!(html.contains("No requirements or rules are available to search."));
    assert!(html.contains("id=\"search-summary\""), "{html}");
    assert!(html.contains("id=\"search-results\""), "{html}");
}

#[test]
fn requirement_domain_classification_links_back_to_its_topic_group() {
    let requirement = corpus_fixture().requirements.remove(0);
    let html = render_requirement("default", &requirement);
    assert!(
        html.contains("href=\"/topics/#domain-dom_invoicing\""),
        "{html}"
    );
}

#[test]
fn search_results_disambiguate_colliding_titles_with_stable_ids() {
    let page = SearchIndexPage {
        scope: "default".to_string(),
        title: "Search".to_string(),
        entries: vec![
            SearchEntry {
                link: link(PageKind::Requirement, "req_alpha_one", "Shared title"),
                kind: PageKind::Requirement,
                statement: "First".to_string(),
            },
            SearchEntry {
                link: link(PageKind::Rule, "rule_alpha_two", "Shared title"),
                kind: PageKind::Rule,
                statement: "Second".to_string(),
            },
        ],
    };
    let html = render_search("default", &page);
    assert!(html.contains("…lpha_one</span>"), "{html}");
    assert!(html.contains("…lpha_two</span>"), "{html}");
}
