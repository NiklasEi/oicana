#import "../src/boxes.typ": *
#import "../src/responsive-image.typ": *
#import "../src/code.typ": *

== Set up C#sym.hash project

#note[This section assumes, that you have a working .NET setup version 8. If that is not the case, please follow #link("https://learn.microsoft.com/en-us/dotnet/core/install/")[the official Microsoft guide] to install .NET on your machine.]

Let's start with a fresh ASP.NET project by executing `dotnet new webapi` in a new directory. The starter project has a single endpoint defined in `Program.cs`. We can test it by starting the service (`dotnet run`) and following the link printed in the terminal. If the page is empty, navigate to `/swagger`.

We want to define a new endpoint to compile our Oicana template to a PDF and return it to the user.

1. Create a new directory in the .NET project called `templates`.
2. Move the packaged `example-0.1.0.zip` into the new directory.
3. Add the `Oicana` nuget package as a dependency (`dotnet add Oicana`).
4. Add the following code to the beginning of `Program.cs`:

  #local-code("Part of Program.cs", "csharp-integration-program-load-file")[
    ```cs
    var file = await File.ReadAllBytesAsync("templates/example-0.1.0.zip");
    var template = new Template(templateFile);
    ```]
5. Define the new endpoint and return the compiled PDF file.

  #local-code(
    "Part of Program.cs",
    "csharp-integration-program-compile-endpoint",
  )[
    ```cs
    app.MapGet("compile", () =>
    {
        var stream = template.Compile([], [], CompilationOptions.Pdf());
        var now = DateTimeOffset.Now;
        return Results.File(
            fileStream: stream,
            contentType: "application/pdf",
            fileDownloadName: $"example_{now:yyyy_MM_dd_HH_mm_ss_ffff}.pdf"
        );
    });
    ```]

  This code defines a new GET endpoint at `/compile`. For every user request, we compile the template with two empty input lists and return the stream as a PDF file.

After restarting the service and refreshing the swagger page, you should see the new endpoint. Open up the endpoint description and click "Try it out" and "Execute" to send a request to the server. If everything works as expected, you should see a successful response with a download button for the PDF file.

#responsive-image("../assets/swagger_response.png")

#note[The PDF generation should not take longer than a couple of milliseconds. You can look at the request duration in the network tab of your browser's debugging tools for an estimation. The first request to an APS.NET service can be significantly slower than later ones, because ASP.NET does some preparation during the first requests by default.

  For a better measurement of the compilation speed on your machine, you can use a #link("https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.stopwatch")[`Stopwatch`] in the endpoint code.
]

We will continue with the C#sym.hash project after adding inputs to the template.
