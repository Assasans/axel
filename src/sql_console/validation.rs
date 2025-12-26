use pg_query::NodeRef;

fn is_forbidden_function_call(func_name: &[pg_query::protobuf::Node]) -> bool {
  for node in func_name {
    if let Some(NodeRef::String(s)) = node.node.as_ref().map(|n| n.to_ref()) {
      if s.sval.eq_ignore_ascii_case("set_config") {
        return true;
      }
    }
  }
  false
}

fn check_node_forbidden(node: NodeRef) -> Option<&'static str> {
  match node {
    NodeRef::VariableSetStmt(stmt) => {
      let kind = stmt.kind();
      match kind {
        pg_query::protobuf::VariableSetKind::VarReset | pg_query::protobuf::VariableSetKind::VarResetAll => {
          Some("RESET statement is not allowed")
        }
        pg_query::protobuf::VariableSetKind::VarSetValue
        | pg_query::protobuf::VariableSetKind::VarSetDefault
        | pg_query::protobuf::VariableSetKind::VarSetCurrent
        | pg_query::protobuf::VariableSetKind::VarSetMulti => {
          if stmt.is_local {
            Some("SET LOCAL statement is not allowed")
          } else {
            Some("SET statement is not allowed")
          }
        }
        _ => None,
      }
    }
    NodeRef::FuncCall(func_call) => {
      if is_forbidden_function_call(&func_call.funcname) {
        Some("set_config() function call is not allowed")
      } else {
        None
      }
    }
    _ => None,
  }
}

fn check_descendants(node: NodeRef) -> Option<&'static str> {
  if let Some(reason) = check_node_forbidden(node) {
    return Some(reason);
  }

  match node {
    NodeRef::SelectStmt(stmt) => {
      for target in &stmt.target_list {
        if let Some(n) = &target.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      for from in &stmt.from_clause {
        if let Some(n) = &from.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      if let Some(where_clause) = &stmt.where_clause {
        if let Some(n) = &where_clause.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      for cte in &stmt.with_clause.iter().flat_map(|w| &w.ctes).collect::<Vec<_>>() {
        if let Some(n) = &cte.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::ResTarget(target) => {
      if let Some(val) = &target.val {
        if let Some(n) = &val.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::FuncCall(func) => {
      for arg in &func.args {
        if let Some(n) = &arg.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::AExpr(expr) => {
      if let Some(lexpr) = &expr.lexpr {
        if let Some(n) = &lexpr.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      if let Some(rexpr) = &expr.rexpr {
        if let Some(n) = &rexpr.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::SubLink(sublink) => {
      if let Some(subselect) = &sublink.subselect {
        if let Some(n) = &subselect.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::CommonTableExpr(cte) => {
      if let Some(query) = &cte.ctequery {
        if let Some(n) = &query.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::RangeSubselect(range) => {
      if let Some(subquery) = &range.subquery {
        if let Some(n) = &subquery.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::InsertStmt(stmt) => {
      if let Some(select) = &stmt.select_stmt {
        if let Some(n) = &select.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::UpdateStmt(stmt) => {
      for target in &stmt.target_list {
        if let Some(n) = &target.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      if let Some(where_clause) = &stmt.where_clause {
        if let Some(n) = &where_clause.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::DeleteStmt(stmt) => {
      if let Some(where_clause) = &stmt.where_clause {
        if let Some(n) = &where_clause.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::CoalesceExpr(expr) => {
      for arg in &expr.args {
        if let Some(n) = &arg.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::CaseExpr(expr) => {
      for arg in &expr.args {
        if let Some(n) = &arg.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      if let Some(defresult) = &expr.defresult {
        if let Some(n) = &defresult.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    NodeRef::CaseWhen(when) => {
      if let Some(expr) = &when.expr {
        if let Some(n) = &expr.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
      if let Some(result) = &when.result {
        if let Some(n) = &result.node {
          if let Some(reason) = check_descendants(n.to_ref()) {
            return Some(reason);
          }
        }
      }
    }
    _ => {}
  }

  None
}

pub fn validate_sql(sql: &str) -> Result<(), String> {
  let result = pg_query::parse(sql).map_err(|e| format!("Parse error: {}", e))?;

  for node in result.protobuf.stmts.iter() {
    if let Some(stmt) = &node.stmt {
      if let Some(inner) = &stmt.node {
        if let Some(reason) = check_node_forbidden(inner.to_ref()) {
          return Err(reason.to_string());
        }
        if let Some(reason) = check_descendants(inner.to_ref()) {
          return Err(reason.to_string());
        }
      }
    }
  }

  Ok(())
}
