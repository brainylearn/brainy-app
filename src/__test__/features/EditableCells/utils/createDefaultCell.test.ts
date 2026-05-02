import createDefaultCell from "../../../../features/EditableCells/utils/createDefaultCell";
import CreateCellRequestDto from "../../../../api/cells/dto/createCellRequestDto";

describe("createDefaultCell", () => {
	it("Note", () => {
		// Arrange

		const fileId = "2";
		const index = 3;
		const expected: CreateCellRequestDto = {
			cellType: "Note",
			content: "",
			fileId,
			index,
		};

		// Act

		const actual = createDefaultCell("Note", fileId, index);

		// Assert

		expect(actual).toStrictEqual(expected);
	});
});
