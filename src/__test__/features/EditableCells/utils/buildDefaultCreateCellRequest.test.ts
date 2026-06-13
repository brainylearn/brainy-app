import buildDefaultCreateCellRequest from "../../../../features/EditableCells/utils/buildDefaultCreateCellRequest";
import CreateCellRequestDto from "../../../../api/cells/dto/createCellRequestDto";

describe("buildDefaultCreateCellRequest", () => {
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

		const actual = buildDefaultCreateCellRequest("Note", fileId, index);

		// Assert

		expect(actual).toStrictEqual(expected);
	});
});
