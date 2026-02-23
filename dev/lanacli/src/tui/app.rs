use ratatui::widgets::{ListState, TableState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Prospects,
    Customers,
    DepositAccounts,
    TermsTemplates,
    CreditFacilities,
    CreditFacilityProposals,
    ApprovalProcesses,
}

pub const ALL_DOMAINS: &[Domain] = &[
    Domain::Prospects,
    Domain::Customers,
    Domain::DepositAccounts,
    Domain::TermsTemplates,
    Domain::CreditFacilities,
    Domain::CreditFacilityProposals,
    Domain::ApprovalProcesses,
];

impl Domain {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Prospects => "Prospects",
            Self::Customers => "Customers",
            Self::DepositAccounts => "Deposit Accounts",
            Self::TermsTemplates => "Terms Templates",
            Self::CreditFacilities => "Credit Facilities",
            Self::CreditFacilityProposals => "Credit Facility Proposals",
            Self::ApprovalProcesses => "Approval Processes",
        }
    }

    pub fn has_detail_query(&self) -> bool {
        !matches!(self, Self::TermsTemplates | Self::CreditFacilityProposals)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Approve,
    Deny,
    Convert,
    Close,
}

impl Action {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Approve => "[a]pprove",
            Self::Deny => "[d]eny",
            Self::Convert => "[c]onvert",
            Self::Close => "[x] close",
        }
    }

    pub fn key(&self) -> char {
        match self {
            Self::Approve => 'a',
            Self::Deny => 'd',
            Self::Convert => 'c',
            Self::Close => 'x',
        }
    }

    pub fn needs_input(&self) -> bool {
        matches!(self, Self::Deny)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    DomainMenu,
    ListView,
    DetailView,
}

pub struct ListData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub ids: Vec<String>,
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

impl ListData {
    pub fn empty() -> Self {
        Self {
            headers: vec![],
            rows: vec![],
            ids: vec![],
            has_next_page: false,
            end_cursor: None,
        }
    }
}

pub struct DetailData {
    pub pairs: Vec<(String, String)>,
    pub actions: Vec<Action>,
    pub entity_id: String,
}

impl DetailData {
    pub fn empty() -> Self {
        Self {
            pairs: vec![],
            actions: vec![],
            entity_id: String::new(),
        }
    }
}

pub struct ListResult {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub ids: Vec<String>,
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

pub struct DetailResult {
    pub pairs: Vec<(String, String)>,
    pub actions: Vec<Action>,
}

pub enum AsyncResult {
    ListLoaded(Domain, anyhow::Result<ListResult>),
    DetailLoaded(anyhow::Result<DetailResult>),
    ActionDone(anyhow::Result<String>),
}

pub struct PendingAction {
    pub action: Action,
    pub entity_id: String,
    pub domain: Domain,
}

pub struct App {
    pub screen: Screen,
    pub menu_state: ListState,
    pub current_domain: Option<Domain>,

    pub list: ListData,
    pub table_state: TableState,

    pub detail: DetailData,
    pub detail_scroll: u16,

    pub status: String,
    pub loading: bool,

    pub input_mode: bool,
    pub input_buffer: String,
    pub pending_action: Option<PendingAction>,
}

impl App {
    pub fn new() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self {
            screen: Screen::DomainMenu,
            menu_state,
            current_domain: None,
            list: ListData::empty(),
            table_state: TableState::default(),
            detail: DetailData::empty(),
            detail_scroll: 0,
            status: String::from("Navigate with arrows, Enter to select, q to quit"),
            loading: false,
            input_mode: false,
            input_buffer: String::new(),
            pending_action: None,
        }
    }

    pub fn selected_domain(&self) -> Domain {
        let idx = self.menu_state.selected().unwrap_or(0);
        ALL_DOMAINS[idx]
    }

    pub fn enter_list(&mut self, domain: Domain, result: ListResult) {
        self.current_domain = Some(domain);
        self.list = ListData {
            headers: result.headers,
            rows: result.rows,
            ids: result.ids,
            has_next_page: result.has_next_page,
            end_cursor: result.end_cursor,
        };
        self.table_state = TableState::default();
        if !self.list.rows.is_empty() {
            self.table_state.select(Some(0));
        }
        self.screen = Screen::ListView;
        self.loading = false;
        let count = self.list.rows.len();
        let more = if self.list.has_next_page {
            " (more available, press n)"
        } else {
            ""
        };
        self.status = format!(
            "{count} {}{more} | Enter=detail  r=refresh  Esc=back",
            domain.label()
        );
    }

    pub fn enter_detail(&mut self, entity_id: String, result: DetailResult) {
        self.detail = DetailData {
            pairs: result.pairs,
            actions: result.actions,
            entity_id,
        };
        self.detail_scroll = 0;
        self.screen = Screen::DetailView;
        self.loading = false;
        let action_hints: Vec<&str> = self.detail.actions.iter().map(|a| a.label()).collect();
        if action_hints.is_empty() {
            self.status = String::from("Esc=back");
        } else {
            self.status = format!("{}  Esc=back", action_hints.join("  "));
        }
    }

    pub fn enter_detail_from_list_row(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.list.rows.len() {
                let pairs = self
                    .list
                    .headers
                    .iter()
                    .zip(self.list.rows[selected].iter())
                    .map(|(h, v)| (h.clone(), v.clone()))
                    .collect();
                let entity_id = self.list.ids[selected].clone();
                self.detail = DetailData {
                    pairs,
                    actions: vec![],
                    entity_id,
                };
                self.detail_scroll = 0;
                self.screen = Screen::DetailView;
                self.status = String::from("Esc=back");
            }
        }
    }

    pub fn handle_async_result(&mut self, result: AsyncResult) {
        match result {
            AsyncResult::ListLoaded(domain, Ok(list_result)) => {
                self.enter_list(domain, list_result);
            }
            AsyncResult::ListLoaded(_, Err(e)) => {
                self.loading = false;
                self.status = format!("Error: {e}");
            }
            AsyncResult::DetailLoaded(Ok(detail_result)) => {
                let entity_id = if let Some(selected) = self.table_state.selected() {
                    self.list.ids.get(selected).cloned().unwrap_or_default()
                } else {
                    String::new()
                };
                self.enter_detail(entity_id, detail_result);
            }
            AsyncResult::DetailLoaded(Err(e)) => {
                self.loading = false;
                self.status = format!("Error: {e}");
            }
            AsyncResult::ActionDone(Ok(msg)) => {
                self.loading = false;
                self.status = msg;
                self.screen = Screen::ListView;
            }
            AsyncResult::ActionDone(Err(e)) => {
                self.loading = false;
                self.status = format!("Action failed: {e}");
            }
        }
    }
}
