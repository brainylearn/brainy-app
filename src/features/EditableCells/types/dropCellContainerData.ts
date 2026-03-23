export const DROP_CELL_CONTAINER_TYPE = "DROP_CELL_CONTAINER_TYPE";

// TODO: use same conventions for file tree
type DropCellContainerData =
	| {
			type: "cell";
			cellId: string;
	  }
	| {
			type: "add-cell-container";
	  };

export default DropCellContainerData;
