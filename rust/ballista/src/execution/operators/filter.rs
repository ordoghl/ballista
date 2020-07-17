// Copyright 2020 Andy Grove
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Filter operator.

use std::sync::Arc;

use crate::arrow::datatypes::Schema;
use crate::datafusion::logicalplan::Expr;
use crate::error::{ballista_error, Result};
use crate::execution::physical_plan::{
    compile_expression, ColumnarBatch, ColumnarBatchIter, ColumnarBatchStream, ColumnarValue,
    ExecutionContext, ExecutionPlan, Expression, PhysicalPlan,
};

use async_trait::async_trait;

/// FilterExec evaluates a boolean expression against each row of input to determine which rows
/// to include in output batches.
#[derive(Debug, Clone)]
pub struct FilterExec {
    pub(crate) child: Arc<PhysicalPlan>,
    filter_expr: Arc<Expr>,
}

#[async_trait]
impl ExecutionPlan for FilterExec {
    fn schema(&self) -> Arc<Schema> {
        unimplemented!()
    }

    fn children(&self) -> Vec<Arc<PhysicalPlan>> {
        vec![self.child.clone()]
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ExecutionContext>,
        partition_index: usize,
    ) -> Result<ColumnarBatchStream> {
        let expr = compile_expression(&self.filter_expr, &self.schema())?;
        Ok(Arc::new(FilterIter {
            input: self
                .child
                .as_execution_plan()
                .execute(ctx.clone(), partition_index)
                .await?,
            filter_expr: expr,
        }))
    }
}

#[allow(dead_code)]
struct FilterIter {
    input: ColumnarBatchStream,
    filter_expr: Arc<dyn Expression>,
}

#[async_trait]
impl ColumnarBatchIter for FilterIter {
    fn schema(&self) -> Arc<Schema> {
        self.input.schema()
    }

    async fn next(&self) -> Result<Option<ColumnarBatch>> {
        match self.input.next().await? {
            Some(input) => {
                let bools = self.filter_expr.evaluate(&input)?;
                Ok(Some(apply_filter(&input, &bools)?))
            }
            None => Ok(None),
        }
    }
}

/// Filter the provided batch based on the bitmask
fn apply_filter(_batch: &ColumnarBatch, _bitmask: &ColumnarValue) -> Result<ColumnarBatch> {
    Err(ballista_error("filter operator is not implemented yet"))
}