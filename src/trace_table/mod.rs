use winterfell::{
    math::{FieldElement, StarkField},
    matrix::ColMatrix,
    EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

#[allow(clippy::uninit_vec)]
pub unsafe fn uninit_vector<T>(length: usize) -> Vec<T> {
    let mut vector = Vec::with_capacity(length);
    vector.set_len(length);
    vector
}

pub struct PermutationTraceTable<B: StarkField> {
    layout: TraceLayout,
    trace: ColMatrix<B>,
    meta: Vec<u8>,
}

impl<B: StarkField> PermutationTraceTable<B> {
    // new execution trace of specified with and length
    // all memory is allocated, but not initialized

    pub fn new(width: usize, length: usize) -> Self {
        Self::with_meta(width, length, vec![])
    }

    //creates new execution trace with specified width and length, but does not initialize it. It is expected that trace will be filled using one of the data mutator methods.

    pub fn with_meta(width: usize, length: usize, meta: Vec<u8>) -> Self {
        assert!(
            width > 0,
            "execution trace must consist of at least one column"
        );
        assert!(
            width <= TraceInfo::MAX_TRACE_WIDTH,
            "execution trace can not be greater than {}, but was {}",
            TraceInfo::MAX_TRACE_WIDTH,
            width
        );
        assert!(
            length.is_power_of_two(),
            "execution trace length must be a power of 2"
        );
        assert!(
            length.ilog2() <= B::TWO_ADICITY,
            "execution trace length cannot exceed 2^{} steps, given 2^{}",
            B::TWO_ADICITY,
            length.ilog2()
        );
        assert!(
            meta.len() <= TraceInfo::MAX_META_LENGTH,
            "meta data bytes exceded the limit"
        );
        // let aux_widths = [1];
        // let aux_rands = [1];
        let columns = unsafe { (0..width).map(|_| uninit_vector(length)).collect() };
        Self {
            layout: TraceLayout::new(width, [1], [1]),
            trace: ColMatrix::new(columns),
            meta,
        }
    }

    // DATA Mutators

    // fill all rows in execution trace
    // init --> initializes first row of trace; receives a mutable reference to the first state initialized to all zeros.
    // contents of the state are copied into the first row of trace after closure returns

    pub fn fill<I, U>(&mut self, init: I, update: U)
    where
        I: Fn(&mut [B]),
        U: Fn(usize, &mut [B]),
    {
        let mut state = vec![B::ZERO; self.main_trace_width()];
        init(&mut state);
        self.update_row(0, &state);

        for i in 0..self.length() - 1 {
            update(i, &mut state);
            self.update_row(i + 1, &state);
        }
    }

    /// Updates a single row in the execution trace with provided data.
    pub fn update_row(&mut self, step: usize, state: &[B]) {
        self.trace.update_row(step, state);
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of columns in this execution trace.
    pub fn width(&self) -> usize {
        self.main_trace_width()
    }

    /// Returns value of the cell in the specified column at the specified row of this trace.
    pub fn get(&self, column: usize, step: usize) -> B {
        self.trace.get(column, step)
    }

    /// Reads a single row from this execution trace into the provided target.
    pub fn read_row_into(&self, step: usize, target: &mut [B]) {
        self.trace.read_row_into(step, target);
    }
}

impl<B: StarkField> Trace for PermutationTraceTable<B> {
    type BaseField = B;

    fn layout(&self) -> &TraceLayout {
        &self.layout
    }

    fn length(&self) -> usize {
        self.trace.num_rows()
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        let next_row_idx = (row_idx + 1) % self.length();
        self.trace.read_row_into(row_idx, frame.current_mut());
        self.trace.read_row_into(next_row_idx, frame.next_mut());
    }

    fn main_segment(&self) -> &ColMatrix<B> {
        &self.trace
    }

    fn build_aux_segment<E>(
        &mut self,
        aux_segments: &[ColMatrix<E>],
        rand_elements: &[E],
    ) -> Option<ColMatrix<E>>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        if !aux_segments.is_empty() {
            return None;
        }

        let mut current_row = unsafe { uninit_vector(self.width()) };
        let mut aux_columns = vec![vec![E::ZERO; self.length()]; 1];
        // why did the other implementations use uninit vec??
        self.read_row_into(0, &mut current_row);
        aux_columns[0][0] = (<B as Into<E>>::into(current_row[0]) + rand_elements[0])
            * (<B as Into<E>>::into(current_row[1]) + rand_elements[0]).inv();

        // aux_columns[0][1] = E::ONE;
        // read row and build aux trace -> [c][r]
        for index in 1..self.length() {
            self.read_row_into(index, &mut current_row);

            let num = <B as Into<E>>::into(current_row[0]) + rand_elements[0];
            let denom = <B as Into<E>>::into(current_row[1]) + rand_elements[0];
            let this_aux = num * denom.inv();
            aux_columns[0][index] = aux_columns[0][index - 1] * this_aux;
        }

        Some(ColMatrix::new(aux_columns))
    }
}
