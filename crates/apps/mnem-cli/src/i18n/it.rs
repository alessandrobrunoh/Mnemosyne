use super::Messages;

pub struct Italian;

impl Messages for Italian {
    fn app_name(&self) -> &'static str {
        "MNEMOSYNE CLI"
    }
    fn tagline(&self) -> &'static str {
        "Il compagno di cronologia locale per sviluppatori."
    }
    fn core_ops_header(&self) -> &'static str {
        "OPERAZIONI PRINCIPALI"
    }

    fn cmd_default_desc(&self) -> &'static str {
        "Avvia la ricerca interattiva TUI e la timeline"
    }
    fn cmd_start_desc(&self) -> &'static str {
        "Assicura che il demone in background sia attivo"
    }
    fn cmd_stop_desc(&self) -> &'static str {
        "Arresta il demone in background"
    }

    fn project_history_header(&self) -> &'static str {
        "STORIA DEL PROGETTO"
    }
    fn cmd_list_desc(&self) -> &'static str {
        "Elenca tutti i progetti tracciati"
    }
    fn cmd_log_desc(&self) -> &'static str {
        "Mostra cronologia/snapshot per un file specifico"
    }
    fn cmd_search_desc(&self) -> &'static str {
        "Grep globale in tutta la storia del progetto"
    }

    fn maintenance_header(&self) -> &'static str {
        "MANUTENZIONE"
    }
    fn cmd_status_desc(&self) -> &'static str {
        "Controlla salute e attività del demone"
    }
    fn cmd_config_desc(&self) -> &'static str {
        "Modifica impostazioni (ritenzione, compressione)"
    }

    fn learn_more(&self) -> &'static str {
        "Per saperne di più, controlla la documentazione"
    }
}
