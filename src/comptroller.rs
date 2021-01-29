pub struct Comptroller {
    stop: bool
}

type ARComptroller = Arc<RefCell<Comptroller>>;

impl Comptroller {

    async fn new() -> ARComptroller
