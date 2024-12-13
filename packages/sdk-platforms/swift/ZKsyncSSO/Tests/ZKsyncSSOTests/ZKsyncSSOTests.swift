import Testing
@testable import ZKsyncSSO

@Test func testGreetRust() async throws {
    #expect(greetRust(name: "Rust") == "Hello, Rust")
}

@Test func testAddRust() async throws {
    #expect(addRust(left: 2, right: 2) == 4)
}
