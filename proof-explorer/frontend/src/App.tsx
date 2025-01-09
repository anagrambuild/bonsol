import { ChakraProvider, Box, Container } from '@chakra-ui/react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from 'react-query';
import RequestList from './components/RequestList';
import RequestDetails from './components/RequestDetails';
import Navbar from './components/Navbar';
import theme from './theme';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchInterval: 5000,
      staleTime: 5000,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ChakraProvider theme={theme}>
        <Router>
          <Box minH="100vh" bg="gray.50">
            <Navbar />
            <Container maxW="container.xl" py={8}>
              <Routes>
                <Route path="/" element={<RequestList />} />
                <Route path="/request/:id" element={<RequestDetails />} />
              </Routes>
            </Container>
          </Box>
        </Router>
      </ChakraProvider>
    </QueryClientProvider>
  );
}

export default App;
